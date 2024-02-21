use std::ops::Deref;
use crate::rocket::BBoxRocket;

pub struct BBoxClient {
    client: rocket::local::blocking::Client,
}

impl BBoxClient {
    // Tracks cookies.
    pub fn tracked<P: rocket::Phase>(rocket: BBoxRocket<P>) -> Result<BBoxClient, rocket::Error> {
        let client = rocket::local::blocking::Client::tracked(rocket.get())?;
        Ok(BBoxClient { client })
    }
    // Does not track cookies (like incognito mode).
    pub fn untracked<P: rocket::Phase>(rocket: BBoxRocket<P>) -> Result<BBoxClient, rocket::Error> {
        let client = rocket::local::blocking::Client::untracked(rocket.get())?;
        Ok(BBoxClient { client })
    }
}

// Deref to use testing functions.
// Example:
// *client.get("/url")
impl Deref for BBoxClient {
    type Target = rocket::local::blocking::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
