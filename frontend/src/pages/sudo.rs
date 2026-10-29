//! The password prompt a job that needs root waits behind (`shell.qml`'s "Administrator
//! password" box): open while `Jobs` has a prompt, with the password field, why the last try
//! failed, and Cancel and Continue. While `sudo.enable` runs it cannot be closed: Escape still
//! asks, as ODK wants a dialog to have a way out, and `Jobs::cancel` does nothing until it ends.

use odk::prelude::*;

use crate::jobs::Jobs;

pub fn prompt(jobs: &Entity<Jobs>, cx: &App) -> impl IntoElement {
    let state = jobs.read(cx).prompt().cloned();
    let minutes = jobs.read(cx).minutes;
    let open = state.is_some();
    let state = state.unwrap_or_default();
    let busy = state.enabling;
    let description = if busy {
        "Unlocking sudo…".to_owned()
    } else {
        format!(
            "This change needs your password. Sudo then stays unlocked for {minutes} minutes. Agents and anything else running as you can use it until then."
        )
    };
    let field = {
        let (typed, submitted) = (jobs.clone(), jobs.clone());
        let (password, error) = (state.password.clone(), state.error.clone());
        move |_: &mut Window, _: &mut App| {
            let (typed, submitted) = (typed.clone(), submitted.clone());
            v_flex()
                .gap(Space::Sm)
                .child(
                    PasswordInput::new("sudo-password", "Password")
                        .placeholder("Password")
                        .value(password.clone())
                        .disabled(busy)
                        .on_change(move |text, _, cx| {
                            typed.update(cx, |jobs, cx| jobs.type_password(text, cx))
                        })
                        .on_submit(move |_, window, cx| {
                            submitted.update(cx, |jobs, cx| jobs.confirm(window, cx))
                        }),
                )
                .children(
                    (!error.is_empty())
                        .then(|| Alert::new("sudo-error", error.clone()).variant(Tone::Danger)),
                )
        }
    };
    let (closed, cancelled, confirmed) = (jobs.clone(), jobs.clone(), jobs.clone());
    Dialog::new("sudo-prompt", "Administrator password")
        .open(open)
        .description(description)
        .close_on_backdrop(!busy)
        .close_button(!busy)
        .on_close(move |_, cx| closed.update(cx, |jobs, cx| jobs.cancel(cx)))
        .content(field)
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("sudo-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .disabled(busy)
                .on_click(move |_, _, cx| cancelled.update(cx, |jobs, cx| jobs.cancel(cx)))
        })
        .action(move |_, _| {
            let confirmed = confirmed.clone();
            Button::new("sudo-continue")
                .label("Continue")
                .variant(ButtonVariant::Primary)
                .loading(busy)
                .on_click(move |_, window, cx| {
                    confirmed.update(cx, |jobs, cx| jobs.confirm(window, cx))
                })
        })
}
