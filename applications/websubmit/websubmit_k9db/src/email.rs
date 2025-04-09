use lettre::sendmail::SendmailTransport;
use lettre::Transport;
use lettre_email::Email;

pub(crate) fn send(
    _log: slog::Logger,
    sender: String,
    recipients: Vec<String>,
    subject: String,
    text: String,
) -> Result<(), lettre::sendmail::error::Error> {
    let mut mailer = SendmailTransport::new();

    let mut builder = Email::builder()
        .from(sender.clone())
        .subject(subject.clone())
        .text(text.clone());
    for recipient in recipients {
        builder = builder.to(recipient);
    }

    // debug!(log, "Email: Subject {}\nText: {}!", subject, text);

    let email = builder.build();
    match email {
        Ok(result) => mailer.send(result.into())?,
        Err(e) => {
            println!("couldn't construct email: {}", e);
        }
    }

    Ok(())
}
