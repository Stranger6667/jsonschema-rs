use referencing::Draft;
use serde_json::{Map, Value};

use crate::{
    compiler,
    error::no_error,
    node::SchemaNode,
    paths::{LazyLocation, Location},
    validator::Validate,
    ErrorIterator, ValidationError,
};

use super::CompilationResult;

pub(crate) trait ItemsFilter: Send + Sync + Sized + 'static {
    fn new<'a>(
        ctx: &compiler::Context<'_>,
        parent: &'a Map<String, Value>,
    ) -> Result<Self, ValidationError<'a>>;
    fn unevaluated(&self) -> Option<&SchemaNode>;

    fn is_valid(&self, instance: &Value) -> bool {
        self.unevaluated()
            .as_ref()
            .map(|u| u.is_valid(instance))
            .unwrap_or(false)
    }

    fn mark_evaluated_indexes(&self, instance: &Value, indexes: &mut Vec<bool>);
}

pub(crate) struct UnevaluatedItemsValidator<F: ItemsFilter> {
    location: Location,
    filter: F,
}

impl<F: ItemsFilter> UnevaluatedItemsValidator<F> {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        parent: &'a Map<String, Value>,
    ) -> CompilationResult<'a> {
        Ok(Box::new(UnevaluatedItemsValidator {
            location: ctx.location().join("unevaluatedItems"),
            filter: F::new(ctx, parent)?,
        }))
    }
}

impl<F: ItemsFilter> Validate for UnevaluatedItemsValidator<F> {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            // NOTE: It could be a packed bitset instead
            let mut indexes = vec![false; items.len()];
            self.filter.mark_evaluated_indexes(instance, &mut indexes);

            for (item, is_evaluated) in items.iter().zip(indexes) {
                if !is_evaluated && !self.filter.is_valid(item) {
                    return false;
                }
            }
        }
        true
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &LazyLocation,
    ) -> ErrorIterator<'instance> {
        if let Value::Array(items) = instance {
            // NOTE: It could be a packed bitset instead
            let mut indexes = vec![false; items.len()];
            self.filter.mark_evaluated_indexes(instance, &mut indexes);
            let mut unevaluated = vec![];
            for (item, is_evaluated) in items.iter().zip(indexes) {
                if !is_evaluated && !self.filter.is_valid(item) {
                    unevaluated.push(item.to_string());
                }
            }
            if !unevaluated.is_empty() {
                return Box::new(
                    vec![ValidationError::unevaluated_items(
                        self.location.clone(),
                        instance_path.into(),
                        instance,
                        unevaluated,
                    )]
                    .into_iter(),
                );
            }
        }
        no_error()
    }
}

struct Draft2019ItemsFilter {
    unevaluated: Option<SchemaNode>,
    contains: Option<SchemaNode>,
    items: Option<usize>,
    conditional: Option<Box<ConditionalFilter<Self>>>,
    all_of: Option<CombinatorFilter<Self>>,
    any_of: Option<CombinatorFilter<Self>>,
    one_of: Option<CombinatorFilter<Self>>,
}

impl ItemsFilter for Draft2019ItemsFilter {
    fn new<'a>(
        ctx: &compiler::Context<'_>,
        parent: &'a Map<String, Value>,
    ) -> Result<Self, ValidationError<'a>> {
        let mut conditional = None;

        if let Some(subschema) = parent.get("if") {
            if let Value::Object(if_parent) = subschema {
                let mut then_ = None;
                if let Some(Value::Object(subschema)) = parent.get("then") {
                    then_ = Some(Self::new(ctx, subschema)?);
                }
                let mut else_ = None;
                if let Some(Value::Object(subschema)) = parent.get("else") {
                    else_ = Some(Self::new(ctx, subschema)?);
                }
                conditional = Some(Box::new(ConditionalFilter {
                    condition: compiler::compile(ctx, ctx.as_resource_ref(subschema))?,
                    if_: Self::new(ctx, if_parent)?,
                    then_,
                    else_,
                }));
            }
        }

        let mut contains = None;
        if let Some(subschema) = parent.get("contains") {
            contains = Some(compiler::compile(ctx, ctx.as_resource_ref(subschema))?);
        };
        let mut unevaluated = None;
        if let Some(subschema) = parent.get("unevaluatedItems") {
            unevaluated = Some(compiler::compile(ctx, ctx.as_resource_ref(subschema))?);
        };
        let mut all_of = None;
        if let Some(Some(subschemas)) = parent.get("allOf").map(Value::as_array) {
            all_of = Some(CombinatorFilter::new(ctx, subschemas)?);
        };
        let mut any_of = None;
        if let Some(Some(subschemas)) = parent.get("anyOf").map(Value::as_array) {
            any_of = Some(CombinatorFilter::new(ctx, subschemas)?);
        };
        let mut one_of = None;
        if let Some(Some(subschemas)) = parent.get("oneOf").map(Value::as_array) {
            one_of = Some(CombinatorFilter::new(ctx, subschemas)?);
        };
        let mut items = None;
        if let Some(subschema) = parent.get("items") {
            let limit = if parent.contains_key("additionalItems") || subschema.is_object() {
                usize::MAX
            } else {
                subschema
                    .as_array()
                    .expect("Items are either an object or an array")
                    .len()
            };
            items = Some(limit);
        };

        Ok(Draft2019ItemsFilter {
            unevaluated,
            contains,
            items,
            conditional,
            all_of,
            any_of,
            one_of,
        })
    }
    fn unevaluated(&self) -> Option<&SchemaNode> {
        self.unevaluated.as_ref()
    }
    fn mark_evaluated_indexes(&self, instance: &Value, indexes: &mut Vec<bool>) {
        if let Some(limit) = self.items {
            for idx in indexes.iter_mut().take(limit) {
                *idx = true;
            }
        }
        if let Some(conditional) = &self.conditional {
            conditional.mark_evaluated_indexes(instance, indexes);
        }
        if let Value::Array(items) = instance {
            for (item, is_evaluated) in items.iter().zip(indexes.iter_mut()) {
                if *is_evaluated {
                    continue;
                }
                if let Some(validator) = &self.contains {
                    if validator.is_valid(item) {
                        *is_evaluated = true;
                        continue;
                    }
                }
                if let Some(validator) = &self.unevaluated {
                    if validator.is_valid(item) {
                        *is_evaluated = true;
                    }
                }
            }
        }

        if let Some(combinator) = &self.all_of {
            if combinator
                .subschemas
                .iter()
                .all(|(v, _)| v.is_valid(instance))
            {
                combinator.mark_evaluated_indexes(instance, indexes);
            }
        }

        if let Some(combinator) = &self.any_of {
            if combinator
                .subschemas
                .iter()
                .all(|(v, _)| v.is_valid(instance))
            {
                combinator.mark_evaluated_indexes(instance, indexes);
            }
        }

        if let Some(combinator) = &self.one_of {
            if combinator
                .subschemas
                .iter()
                .filter(|(v, _)| v.is_valid(instance))
                .count()
                == 1
            {
                combinator.mark_evaluated_indexes(instance, indexes);
            }
        }
    }
}

struct DefaultItemsFilter {
    unevaluated: Option<SchemaNode>,
    contains: Option<SchemaNode>,
    items: bool,
    prefix_items: Option<usize>,
    conditional: Option<Box<ConditionalFilter<Self>>>,
    all_of: Option<CombinatorFilter<Self>>,
    any_of: Option<CombinatorFilter<Self>>,
    one_of: Option<CombinatorFilter<Self>>,
}

impl ItemsFilter for DefaultItemsFilter {
    fn new<'a>(
        ctx: &compiler::Context<'_>,
        parent: &'a Map<String, Value>,
    ) -> Result<DefaultItemsFilter, ValidationError<'a>> {
        let mut conditional = None;

        if let Some(subschema) = parent.get("if") {
            if let Value::Object(if_parent) = subschema {
                let mut then_ = None;
                if let Some(Value::Object(subschema)) = parent.get("then") {
                    then_ = Some(DefaultItemsFilter::new(ctx, subschema)?);
                }
                let mut else_ = None;
                if let Some(Value::Object(subschema)) = parent.get("else") {
                    else_ = Some(DefaultItemsFilter::new(ctx, subschema)?);
                }
                conditional = Some(Box::new(ConditionalFilter {
                    condition: compiler::compile(ctx, ctx.as_resource_ref(subschema))?,
                    if_: DefaultItemsFilter::new(ctx, if_parent)?,
                    then_,
                    else_,
                }));
            }
        }

        let mut prefix_items = None;
        if let Some(Some(items)) = parent.get("prefixItems").map(Value::as_array) {
            prefix_items = Some(items.len());
        }

        let mut contains = None;
        if let Some(subschema) = parent.get("contains") {
            contains = Some(compiler::compile(ctx, ctx.as_resource_ref(subschema))?);
        };
        let mut unevaluated = None;
        if let Some(subschema) = parent.get("unevaluatedItems") {
            unevaluated = Some(compiler::compile(ctx, ctx.as_resource_ref(subschema))?);
        };
        let mut all_of = None;
        if let Some(Some(subschemas)) = parent.get("allOf").map(Value::as_array) {
            all_of = Some(CombinatorFilter::new(ctx, subschemas)?);
        };
        let mut any_of = None;
        if let Some(Some(subschemas)) = parent.get("anyOf").map(Value::as_array) {
            any_of = Some(CombinatorFilter::new(ctx, subschemas)?);
        };

        let mut one_of = None;
        if let Some(Some(subschemas)) = parent.get("oneOf").map(Value::as_array) {
            one_of = Some(CombinatorFilter::new(ctx, subschemas)?);
        };

        Ok(DefaultItemsFilter {
            unevaluated,
            contains,
            items: parent.contains_key("items"),
            prefix_items,
            conditional,
            all_of,
            any_of,
            one_of,
        })
    }
    fn unevaluated(&self) -> Option<&SchemaNode> {
        self.unevaluated.as_ref()
    }

    fn mark_evaluated_indexes(&self, instance: &Value, indexes: &mut Vec<bool>) {
        if self.items {
            for idx in indexes {
                *idx = true;
            }
            return;
        }
        if let Some(limit) = self.prefix_items {
            for idx in indexes.iter_mut().take(limit) {
                *idx = true;
            }
        }
        if let Some(conditional) = &self.conditional {
            conditional.mark_evaluated_indexes(instance, indexes);
        }
        if let Value::Array(items) = instance {
            for (item, is_evaluated) in items.iter().zip(indexes.iter_mut()) {
                if *is_evaluated {
                    continue;
                }
                if let Some(validator) = &self.contains {
                    if validator.is_valid(item) {
                        *is_evaluated = true;
                        continue;
                    }
                }
                if let Some(validator) = &self.unevaluated {
                    if validator.is_valid(item) {
                        *is_evaluated = true;
                    }
                }
            }
        }

        if let Some(combinator) = &self.all_of {
            if combinator
                .subschemas
                .iter()
                .all(|(v, _)| v.is_valid(instance))
            {
                combinator.mark_evaluated_indexes(instance, indexes);
            }
        }

        if let Some(combinator) = &self.any_of {
            if combinator
                .subschemas
                .iter()
                .all(|(v, _)| v.is_valid(instance))
            {
                combinator.mark_evaluated_indexes(instance, indexes);
            }
        }

        if let Some(combinator) = &self.one_of {
            if combinator
                .subschemas
                .iter()
                .filter(|(v, _)| v.is_valid(instance))
                .count()
                == 1
            {
                combinator.mark_evaluated_indexes(instance, indexes);
            }
        }
    }
}

struct CombinatorFilter<F> {
    subschemas: Vec<(SchemaNode, F)>,
}

impl<F: ItemsFilter> CombinatorFilter<F> {
    fn mark_evaluated_indexes(&self, instance: &Value, indexes: &mut Vec<bool>) {
        for (_, subschema) in &self.subschemas {
            subschema.mark_evaluated_indexes(instance, indexes);
        }
    }
}

impl<F: ItemsFilter> CombinatorFilter<F> {
    fn new<'a>(
        ctx: &compiler::Context,
        subschemas: &'a [Value],
    ) -> Result<CombinatorFilter<F>, ValidationError<'a>> {
        let mut buffer = Vec::with_capacity(subschemas.len());
        for subschema in subschemas {
            if let Value::Object(parent) = subschema {
                buffer.push((
                    compiler::compile(ctx, ctx.as_resource_ref(subschema))?,
                    F::new(ctx, parent)?,
                ));
            }
        }
        Ok(CombinatorFilter { subschemas: buffer })
    }
}

struct ConditionalFilter<F> {
    condition: SchemaNode,
    if_: F,
    then_: Option<F>,
    else_: Option<F>,
}

impl<F: ItemsFilter> ConditionalFilter<F> {
    fn mark_evaluated_indexes(&self, instance: &Value, indexes: &mut Vec<bool>) {
        if self.condition.is_valid(instance) {
            self.if_.mark_evaluated_indexes(instance, indexes);
            if let Some(then_) = &self.then_ {
                then_.mark_evaluated_indexes(instance, indexes);
            }
        } else if let Some(else_) = &self.else_ {
            else_.mark_evaluated_indexes(instance, indexes);
        }
    }
}

pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    match schema.as_bool() {
        Some(true) => None,
        _ => {
            if ctx.draft() == Draft::Draft201909 {
                Some(UnevaluatedItemsValidator::<Draft2019ItemsFilter>::compile(
                    ctx, parent,
                ))
            } else {
                Some(UnevaluatedItemsValidator::<DefaultItemsFilter>::compile(
                    ctx, parent,
                ))
            }
        }
    }
}
