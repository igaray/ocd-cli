use dialoguer::Confirm;

pub fn user_confirm() -> bool {
    Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()
        .unwrap_or(false)
}
