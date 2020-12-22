use dialoguer::Confirm;

pub fn user_confirm() -> bool {
    match Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()
    {
        Ok(cont) => cont,
        Err(_) => false,
    }
}
