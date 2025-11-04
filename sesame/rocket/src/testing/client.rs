use crate::rocket::SesameRocket;
use std::ops::Deref;

pub struct SesameClient {
    client: rocket::local::blocking::Client,
}

impl SesameClient {
    // Tracks cookies.
    pub fn tracked<P: rocket::Phase>(
        rocket: SesameRocket<P>,
    ) -> Result<SesameClient, rocket::Error> {
        let client = rocket::local::blocking::Client::tracked(rocket.get())?;
        Ok(SesameClient { client })
    }
    // Does not track cookies (like incognito mode).
    pub fn untracked<P: rocket::Phase>(
        rocket: SesameRocket<P>,
    ) -> Result<SesameClient, rocket::Error> {
        let client = rocket::local::blocking::Client::untracked(rocket.get())?;
        Ok(SesameClient { client })
    }
}

// Deref to use testing functions.
// Example:
// *client.get("/url")
impl Deref for SesameClient {
    type Target = rocket::local::blocking::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
