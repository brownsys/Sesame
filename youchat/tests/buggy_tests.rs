use alohomora::testing::BBoxClient;
use rocket::http::Status;
use youchat;
mod common;
use common::{response_contains, random_string, ResponsePortion};
use rocket::http::ContentType;

const INTENDED_BUGGY_ERROR: Status = Status::InternalServerError;

#[test]
fn buggy_endpoint_crashes(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();
    
    // check to make sure we get the intended error on the buggy endpoint
    let response = client.get("/buggy/ali").dispatch();
    assert!(response.status() == INTENDED_BUGGY_ERROR);
    
    // send a random confidential message
    let recipient = random_string(6);
    let user = "abcde".to_string();
    let message = random_string(30);
    let _ = client.post(format!("/chat/{user}/send")).header(ContentType::Form)//.header("application/x-www-form-urlencoded")
            .body(format!("recipient={recipient}&content={}", message.replace(" ", "+"))).dispatch();

    // make sure this causes the buggy endpoint to crash for all users
    for user in ["alex", "daniella", "barry", "charlie"] {
        let response = client.get(format!("/buggy/{user}")).dispatch();
        assert!(response.status() == INTENDED_BUGGY_ERROR);
    }
}

#[test]
fn buggy_endpoint_doesnt_leak_existing(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();
    
    // check to make sure we get the intended error on the buggy endpoint
    let response = client.get("/buggy/ali").dispatch();
    let response_string = response.into_string().unwrap();

    // check to make sure no chat information can be found in response
    // TODO: expand to all chat info (timestamps & senders)
    let private_chats = ["charlie charlie charlie", "(2) what do you want kiddo", "hi barry how are you doing?", "hi alex it's me barry", "(1) i'm at work so i cant talk right now"];
    for private_chat in private_chats {
        assert!(!response_string.contains(private_chat));
    }
}

#[test]
fn buggy_endpoint_doesnt_leak_sent(){
    // initialize client connection
    let serv = youchat::build_server();

    // use alohomora testing client
    // rust rocket web app, list from justus
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    // make three random confidential chats
    let recipient = random_string(6);
    let user = "barry".to_string();
    let private_chats = [random_string(30), random_string(40), random_string(50)];

    // send them to a random recipient
    for private_chat in private_chats.clone() {
        let _ = client.post(format!("/chat/{user}/send")).header(ContentType::Form)//.header("application/x-www-form-urlencoded")
            .body(format!("recipient={recipient}&content={}", private_chat.replace(" ", "+"))).dispatch();
    }

    // all other users shouldn't be able to see anything about the chats
    for user in ["alex", "daniella", "barry", "charlie"] {
        let response = client.get(format!("/buggy/{user}")).dispatch();
        let response_string = response.into_string().unwrap();
        for private_chat in private_chats.clone() {
            assert!(!response_string.contains(&private_chat.clone()));
        }
    }
}

// all chats in the default schema.sql should be accessible to "alex", 
// either through direct messaging or groupchats so the buggy endpoint should be ALLOWED
#[test]
fn buggy_endpoint_works_if_all_chats_allowed(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();
    
    // check to make sure we don't get an error on the buggy endpoint 
    // if we should be able to access all the chats
    let response = client.get("/buggy/alex").dispatch();
    assert!(response.status() == Status::Ok);

    let expected_chats = ["hi alex it's me barry", "hi barry how are you doing?", "charlie charlie charlie", "(2) what do you want kiddo", "(1) i'm at work so i cant talk right now", "hi everyone! welcome to the groupchat 'testgroupchat'!", "thanks! im happy to be here"];

    for chat in expected_chats {
        assert!(response_contains(&client, "/buggy/alex".to_string(), chat, ResponsePortion::RecievedChats));
    }
}