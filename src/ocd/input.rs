use dialoguer::Confirmation;

pub fn user_confirm() -> bool {
    match Confirmation::new()
        .with_text("Do you want to continue?")
        .interact()
    {
        Ok(cont) => cont,
        Err(_) => false,
    }
}
