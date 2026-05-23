use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Save,
    Apply,
    Validate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Args {
    pub action: Action,
    pub profile: String,
}

pub fn get() -> Result<Args> {
    let mut args = std::env::args().skip(1);

    let action = match args.next() {
        Some(arg) if arg == "save" => Action::Save,
        Some(arg) if arg == "apply" => Action::Apply,
        Some(arg) if arg == "validate" => Action::Validate,
        Some(arg) => return Err(ArgumentError::UnexpectedAction(arg)),
        None => return Err(ArgumentError::MissingAction),
    };

    let Some(profile) = args.next() else {
        return Err(ArgumentError::MissingProfile);
    };

    if let Some(arg) = args.next() {
        return Err(ArgumentError::Unexpected(arg));
    }

    Ok(Args { action, profile })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArgumentError {
    UnexpectedAction(String),
    MissingAction,
    MissingProfile,
    Unexpected(String),
}
impl std::error::Error for ArgumentError {}
impl Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgumentError::UnexpectedAction(arg) => {
                write!(f, "unexpected ACTION {arg:?}")
            }
            ArgumentError::MissingAction => {
                write!(f, r#"missing ACTION ("save", "apply" or "validate")"#)
            }
            ArgumentError::MissingProfile => write!(f, "missing profile FILE"),
            ArgumentError::Unexpected(arg) => write!(f, "unexpected: {arg:?}"),
        }
    }
}

type Result<T, E = ArgumentError> = std::result::Result<T, E>;
