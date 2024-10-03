use crate::{
    compiler::Context,
    error::ErrorIterator,
    keywords::BoxedValidator,
    output::{Annotations, BasicOutput, ErrorDescription, OutputUnit},
    paths::{JsonPointer, JsonPointerNode, PathChunk},
    validator::{PartialApplication, Validate},
    ValidationError,
};
use ahash::AHashMap;
use referencing::{
    uri::{self, EncodedString, Path},
    Uri,
};
use std::{collections::VecDeque, fmt};

/// A node in the schema tree, returned by [`compiler::compile`]
#[derive(Debug)]
pub(crate) struct SchemaNode {
    validators: NodeValidators,
    relative_path: JsonPointer,
    absolute_path: Option<Uri<String>>,
}

enum NodeValidators {
    /// The result of compiling a boolean valued schema, e.g
    ///
    /// ```json
    /// {
    ///     "additionalProperties": false
    /// }
    /// ```
    ///
    /// Here the result of `compiler::compile` called with the `false` value will return a
    /// `SchemaNode` with a single `BooleanValidator` as it's `validators`.
    Boolean { validator: Option<BoxedValidator> },
    /// The result of compiling a schema which is composed of keywords (almost all schemas)
    Keyword(Box<KeywordValidators>),
    /// The result of compiling a schema which is "array valued", e.g the "dependencies" keyword of
    /// draft 7 which can take values which are an array of other property names
    Array { validators: Vec<BoxedValidator> },
}

impl fmt::Debug for NodeValidators {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean { .. } => f.debug_struct("Boolean").finish(),
            Self::Keyword(_) => f.debug_tuple("Keyword").finish(),
            Self::Array { .. } => f.debug_struct("Array").finish(),
        }
    }
}

struct KeywordValidators {
    /// The keywords on this node which were not recognized by any vocabularies. These are
    /// stored so we can later produce them as annotations
    unmatched_keywords: Option<AHashMap<String, serde_json::Value>>,
    // We should probably use AHashMap here but it breaks a bunch of test which assume
    // validators are in a particular order
    validators: Vec<(String, BoxedValidator)>,
}

impl SchemaNode {
    pub(crate) fn from_boolean(ctx: &Context<'_>, validator: Option<BoxedValidator>) -> SchemaNode {
        SchemaNode {
            relative_path: ctx.clone().into_pointer(),
            absolute_path: ctx.base_uri(),
            validators: NodeValidators::Boolean { validator },
        }
    }

    pub(crate) fn from_keywords(
        ctx: &Context<'_>,
        mut validators: Vec<(String, BoxedValidator)>,
        unmatched_keywords: Option<AHashMap<String, serde_json::Value>>,
    ) -> SchemaNode {
        validators.shrink_to_fit();
        SchemaNode {
            relative_path: ctx.clone().into_pointer(),
            absolute_path: ctx.base_uri(),
            validators: NodeValidators::Keyword(Box::new(KeywordValidators {
                unmatched_keywords,
                validators,
            })),
        }
    }

    pub(crate) fn from_array(ctx: &Context<'_>, mut validators: Vec<BoxedValidator>) -> SchemaNode {
        validators.shrink_to_fit();
        SchemaNode {
            relative_path: ctx.clone().into_pointer(),
            absolute_path: ctx.base_uri(),
            validators: NodeValidators::Array { validators },
        }
    }

    pub(crate) fn validators(&self) -> impl ExactSizeIterator<Item = &BoxedValidator> {
        match &self.validators {
            NodeValidators::Boolean { validator } => {
                if let Some(v) = validator {
                    NodeValidatorsIter::BooleanValidators(std::iter::once(v))
                } else {
                    NodeValidatorsIter::NoValidator
                }
            }
            NodeValidators::Keyword(kvals) => {
                NodeValidatorsIter::KeywordValidators(kvals.validators.iter())
            }
            NodeValidators::Array { validators } => {
                NodeValidatorsIter::ArrayValidators(validators.iter())
            }
        }
    }

    /// This is similar to `Validate::apply` except that `SchemaNode` knows where it is in the
    /// validator tree and so rather than returning a `PartialApplication` it is able to return a
    /// complete `BasicOutput`. This is the mechanism which compositional validators use to combine
    /// results from sub-schemas
    pub(crate) fn apply_rooted(
        &self,
        instance: &serde_json::Value,
        instance_path: &JsonPointerNode,
    ) -> BasicOutput {
        match self.apply(instance, instance_path) {
            PartialApplication::Valid {
                annotations,
                mut child_results,
            } => {
                if let Some(annotations) = annotations {
                    child_results.insert(0, self.annotation_at(instance_path, annotations));
                };
                BasicOutput::Valid(child_results)
            }
            PartialApplication::Invalid {
                errors,
                mut child_results,
            } => {
                for error in errors {
                    child_results.insert(0, self.error_at(instance_path, error));
                }
                BasicOutput::Invalid(child_results)
            }
        }
    }

    /// Create an error output which is marked as occurring at this schema node
    pub(crate) fn error_at(
        &self,
        instance_path: &JsonPointerNode,
        error: ErrorDescription,
    ) -> OutputUnit<ErrorDescription> {
        OutputUnit::<ErrorDescription>::error(
            self.relative_path.clone(),
            instance_path.clone().into(),
            self.absolute_path.clone(),
            error,
        )
    }

    /// Create an annotation output which is marked as occurring at this schema node
    pub(crate) fn annotation_at<'a>(
        &self,
        instance_path: &JsonPointerNode,
        annotations: Annotations<'a>,
    ) -> OutputUnit<Annotations<'a>> {
        OutputUnit::<Annotations<'_>>::annotations(
            self.relative_path.clone(),
            instance_path.clone().into(),
            self.absolute_path.clone(),
            annotations,
        )
    }

    /// Here we return a `NodeValidatorsErrIter` to avoid allocating in some situations. This isn't
    /// always possible but for a lot of common cases (e.g nodes with a single child) we can do it.
    /// This is wrapped in a `Box` by `SchemaNode::validate`
    pub(crate) fn err_iter<'a>(
        &self,
        instance: &'a serde_json::Value,
        instance_path: &JsonPointerNode,
    ) -> NodeValidatorsErrIter<'a> {
        match &self.validators {
            NodeValidators::Keyword(kvs) if kvs.validators.len() == 1 => {
                NodeValidatorsErrIter::Single(kvs.validators[0].1.validate(instance, instance_path))
            }
            NodeValidators::Keyword(kvs) => NodeValidatorsErrIter::Multiple(
                kvs.validators
                    .iter()
                    .flat_map(|(_, v)| v.validate(instance, instance_path))
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
            NodeValidators::Boolean {
                validator: Some(v), ..
            } => NodeValidatorsErrIter::Single(v.validate(instance, instance_path)),
            NodeValidators::Boolean {
                validator: None, ..
            } => NodeValidatorsErrIter::NoErrs,
            NodeValidators::Array { validators } => NodeValidatorsErrIter::Multiple(
                validators
                    .iter()
                    .flat_map(move |v| v.validate(instance, instance_path))
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
        }
    }

    /// Helper function to apply an iterator of `(Into<PathChunk>, Validate)` to a value. This is
    /// useful as a keyword schemanode has a set of validators keyed by their keywords, so the
    /// `Into<Pathchunk>` is a `String` whereas an array schemanode has an array of validators so
    /// the `Into<PathChunk>` is a `usize`
    fn apply_subschemas<'a, I, P>(
        &self,
        instance: &serde_json::Value,
        instance_path: &JsonPointerNode,
        path_and_validators: I,
        annotations: Option<Annotations<'a>>,
    ) -> PartialApplication<'a>
    where
        I: Iterator<Item = (P, &'a Box<dyn Validate + Send + Sync + 'a>)> + 'a,
        P: Into<PathChunk> + fmt::Display,
    {
        let mut success_results: VecDeque<OutputUnit<Annotations>> = VecDeque::new();
        let mut error_results = VecDeque::new();
        for (path, validator) in path_and_validators {
            let path = self.relative_path.extend_with(&[path.into()]);
            let absolute_keyword_location = self.absolute_path.as_ref().map(|absolute_path| {
                let mut buffer = EncodedString::new();
                for chunk in path.iter() {
                    buffer.push('/');
                    match chunk {
                        PathChunk::Property(p) => buffer.encode::<Path>(p.as_bytes()),
                        PathChunk::Index(i) => {
                            buffer.encode::<Path>(itoa::Buffer::new().format(*i).as_bytes())
                        }
                        PathChunk::Keyword(k) => buffer.encode::<Path>(k.as_bytes()),
                    }
                }
                uri::resolve_against(&absolute_path.borrow(), &format!("#{}", buffer.as_str()))
                    .expect("Invalid reference")
            });
            match validator.apply(instance, instance_path) {
                PartialApplication::Valid {
                    annotations,
                    child_results,
                } => {
                    if let Some(annotations) = annotations {
                        success_results.push_front(OutputUnit::<Annotations<'a>>::annotations(
                            path,
                            instance_path.into(),
                            absolute_keyword_location,
                            annotations,
                        ));
                    }
                    success_results.extend(child_results);
                }
                PartialApplication::Invalid {
                    errors: these_errors,
                    child_results,
                } => {
                    error_results.extend(child_results);
                    error_results.extend(these_errors.into_iter().map(|error| {
                        OutputUnit::<ErrorDescription>::error(
                            path.clone(),
                            instance_path.into(),
                            absolute_keyword_location.clone(),
                            error,
                        )
                    }));
                }
            }
        }
        if error_results.is_empty() {
            PartialApplication::Valid {
                annotations,
                child_results: success_results,
            }
        } else {
            PartialApplication::Invalid {
                errors: Vec::new(),
                child_results: error_results,
            }
        }
    }
}

impl Validate for SchemaNode {
    fn validate<'instance>(
        &self,
        instance: &'instance serde_json::Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        Box::new(self.err_iter(instance, instance_path))
    }

    fn is_valid(&self, instance: &serde_json::Value) -> bool {
        match &self.validators {
            // If we only have one validator then calling it's `is_valid` directly does
            // actually save the 20 or so instructions required to call the `slice::Iter::all`
            // implementation. Validators at the leaf of a tree are all single node validators so
            // this optimization can have significant cumulative benefits
            NodeValidators::Keyword(kvs) if kvs.validators.len() == 1 => {
                kvs.validators[0].1.is_valid(instance)
            }
            NodeValidators::Keyword(kvs) => {
                kvs.validators.iter().all(|(_, v)| v.is_valid(instance))
            }
            NodeValidators::Array { validators } => validators.iter().all(|v| v.is_valid(instance)),
            NodeValidators::Boolean { validator: Some(_) } => false,
            NodeValidators::Boolean { validator: None } => true,
        }
    }

    fn apply<'a>(
        &'a self,
        instance: &serde_json::Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        match self.validators {
            NodeValidators::Array { ref validators } => {
                self.apply_subschemas(instance, instance_path, validators.iter().enumerate(), None)
            }
            NodeValidators::Boolean { ref validator } => {
                if let Some(validator) = validator {
                    validator.apply(instance, instance_path)
                } else {
                    PartialApplication::Valid {
                        annotations: None,
                        child_results: VecDeque::new(),
                    }
                }
            }
            NodeValidators::Keyword(ref kvals) => {
                let KeywordValidators {
                    ref unmatched_keywords,
                    ref validators,
                } = **kvals;
                let annotations: Option<Annotations<'a>> =
                    unmatched_keywords.as_ref().map(Annotations::from);
                self.apply_subschemas(
                    instance,
                    instance_path,
                    validators.iter().map(|(p, v)| (p.clone(), v)),
                    annotations,
                )
            }
        }
    }
}

enum NodeValidatorsIter<'a> {
    NoValidator,
    BooleanValidators(std::iter::Once<&'a BoxedValidator>),
    KeywordValidators(std::slice::Iter<'a, (String, BoxedValidator)>),
    ArrayValidators(std::slice::Iter<'a, BoxedValidator>),
}

impl<'a> Iterator for NodeValidatorsIter<'a> {
    type Item = &'a BoxedValidator;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::NoValidator => None,
            Self::BooleanValidators(i) => i.next(),
            Self::KeywordValidators(v) => v.next().map(|(_, v)| v),
            Self::ArrayValidators(v) => v.next(),
        }
    }

    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        match self {
            Self::NoValidator => true,
            Self::BooleanValidators(i) => i.all(f),
            Self::KeywordValidators(v) => v.all(|(_, v)| f(v)),
            Self::ArrayValidators(v) => v.all(f),
        }
    }
}

impl<'a> ExactSizeIterator for NodeValidatorsIter<'a> {
    fn len(&self) -> usize {
        match self {
            Self::NoValidator => 0,
            Self::BooleanValidators(..) => 1,
            Self::KeywordValidators(v) => v.len(),
            Self::ArrayValidators(v) => v.len(),
        }
    }
}

pub(crate) enum NodeValidatorsErrIter<'a> {
    NoErrs,
    Single(ErrorIterator<'a>),
    Multiple(std::vec::IntoIter<ValidationError<'a>>),
}

impl<'a> Iterator for NodeValidatorsErrIter<'a> {
    type Item = ValidationError<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::NoErrs => None,
            Self::Single(i) => i.next(),
            Self::Multiple(ms) => ms.next(),
        }
    }
}
