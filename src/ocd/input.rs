use dialoguer::Confirmation;

pub fn user_confirm() -> bool {
    Confirmation::new()
        .with_text("Do you want to continue?")
        .interact()
        .unwrap_or(false)
}
