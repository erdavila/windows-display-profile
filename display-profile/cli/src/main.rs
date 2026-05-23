use std::{fs, io};

use anyhow::Result;
use display_profile_lib::SetProfileAction;

use crate::args::Action;

mod args;

fn main() -> Result<()> {
    let args = args::get()?;

    match args.action {
        Action::Save => save(&args.profile)?,
        Action::Apply => apply_or_validate(&args.profile, SetProfileAction::Apply)?,
        Action::Validate => apply_or_validate(&args.profile, SetProfileAction::Validate)?,
    }

    Ok(())
}

fn save(profile_path: &str) -> Result<()> {
    let profile = display_profile_lib::get_profile()?;

    if profile_path == "-" {
        serde_json::to_writer_pretty(io::stdout(), &profile)?;
    } else {
        let file = fs::File::create(profile_path)?;
        serde_json::to_writer_pretty(file, &profile)?;
    }

    Ok(())
}

fn apply_or_validate(profile_path: &str, action: SetProfileAction) -> Result<()> {
    let profile = fs::read_to_string(profile_path)?;
    let profile = serde_json::from_str(&profile)?;
    display_profile_lib::set_profile(&profile, action)?;
    Ok(())
}
