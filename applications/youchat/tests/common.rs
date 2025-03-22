use alohomora::testing::BBoxClient;
use rocket::http::Status;
use rand::{distributions::Alphanumeric, Rng};

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum ResponsePortion{
    Any,
    RecievedChats,
    SentChats
}

#[allow(dead_code)]
// uses "client" to determine if a get request to "uri" will result in a response containing "chat"
// within the specified "portion" of the response
pub(crate) fn response_contains(client: &BBoxClient, uri: String, chat: &str, portion: ResponsePortion)
    -> bool {
    // make sure we get an okay response from the server
    let response = client.get(uri).dispatch();
    assert!(response.status() == Status::Ok);

    // make sure it contains the chat
    let mut response_string = response.into_string().unwrap();

    // modify the section of the response_string we're looking at given the specified portion
    if portion == ResponsePortion::RecievedChats{
        response_string = response_string[response_string.find("Recieved Chats").unwrap()..response_string.len()].to_string();
    } else if portion == ResponsePortion::SentChats{
        response_string = response_string[response_string.find("Sent Chats").unwrap()..response_string.find("Recieved Chats").unwrap()].to_string();
    }

    response_string.contains(chat)
}

#[allow(dead_code)]
// generates a random string of length "len" for use in testing
pub(crate) fn random_string(len: usize) -> String{
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();
    s
}

#[allow(dead_code)]
// tests if a given status is a valid HTTP redirect status
pub(crate) fn is_redirect(status: Status) -> bool{
    (status.code / 100) == 3
}

#[allow(dead_code)]
// uses "client" to check if a get request to "uri" results in a redirect to "desired_dest"
pub(crate) fn response_redirects(client: &BBoxClient, uri: String, desired_dest: &str)
    -> bool {
    // make sure we get redirected (w/ a 3XX HTTP response code)
    let response = client.get(uri).dispatch();
    assert!(is_redirect(response.status()));
    
    // make sure it's to the desired page
    let dest = response.headers().get_one("Location").unwrap().to_string();
    dest == desired_dest
}