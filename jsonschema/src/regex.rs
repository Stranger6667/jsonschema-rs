use std::fmt::Display;
use crate::compilation::options::RegexEngine;

#[derive(Debug)]
pub enum RegexError {
    Regex(regex::Error),
    FancyRegex(fancy_regex::Error),
}

impl Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::Regex(e) => write!(f, "{}", e),
            RegexError::FancyRegex(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub enum Regex {
    Regex(regex::Regex),
    FancyRegex(fancy_regex::Regex),
}

impl Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Regex::Regex(r) => write!(f, "{}", r),
            Regex::FancyRegex(r) => write!(f, "{}", r),
        }
    }
}

impl Regex {
    pub fn new(pattern: &str, regex_engine: &RegexEngine) -> Result<Self, RegexError> {
        match regex_engine {
            RegexEngine::Regex(options) => {
                let mut builder  = &mut regex::RegexBuilder::new(pattern);

                if let Some(size_limit) = options.size_limit {
                    builder = builder.size_limit(size_limit);
                }

                if let Some(dfa_size_limit) = options.dfa_size_limit {
                    builder = builder.dfa_size_limit(dfa_size_limit);
                }

                builder.build()
                    .map(Regex::Regex)
                    .map_err(RegexError::Regex)
            },
            RegexEngine::FancyRegex(options) => {
                let mut builder  = &mut fancy_regex::RegexBuilder::new(pattern);

                if let Some(backtrack_limit) = options.backtrack_limit {
                    builder = builder.backtrack_limit(backtrack_limit);
                }

                if let Some(delegate_size_limit) = options.delegate_size_limit {
                    builder = builder.delegate_size_limit(delegate_size_limit);
                }

                if let Some(delegate_dfa_size_limit) = options.delegate_dfa_size_limit {
                    builder = builder.delegate_dfa_size_limit(delegate_dfa_size_limit);
                }

                builder.build()
                    .map(Regex::FancyRegex)
                    .map_err(RegexError::FancyRegex)
            },
        }
    }

    pub fn is_match(&self, text: &str) -> Result<bool, RegexError> {
        match self {
            Regex::Regex(r) => Ok(r.is_match(text)),
            Regex::FancyRegex(r) => r.is_match(text).map_err(RegexError::FancyRegex),
        }
    }
}
