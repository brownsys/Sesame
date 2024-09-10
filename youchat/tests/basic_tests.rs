//use rocket::local::asynchronous::Client;
use alohomora::testing::BBoxClient;
use rocket::http::ContentType;
use youchat;
mod common;
use common::{response_contains, response_redirects, is_redirect, random_string, ResponsePortion};

#[test]
fn users_can_connect(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();
    
    // check to make sure we're redirected from root to login
    assert!(response_redirects(&client, "/".to_string(), "/login"));

    // check that the login page renders
    assert!(response_contains(&client, "/login".to_string(), "<h1>Welcome to YouChat!</h1>", ResponsePortion::Any));
}

#[test]
fn users_can_send_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let users = ["alex", "barry", "cher"];
    let messages_from_each_user = 6;
    // loop through users and send messages from each of them to random users
    for user in users {
        for _ in 0..messages_from_each_user {
            // send random message to random recipient
            let recipient = random_string(6);
            let message = random_string(30);

            let response = client.post(format!("/chat/{user}/send")).header(ContentType::Form)//.header("application/x-www-form-urlencoded")
                            .body(format!("recipient={recipient}&content={}", message.replace(" ", "+"))).dispatch();
            
            // make sure we get redirected
            assert!(is_redirect(response.status()));

            // make sure we can see it in our sent
            assert!(response_contains(&client, format!("/chat/{user}"), &message, ResponsePortion::SentChats));
        }
    }
}

#[test]
fn users_can_recieve_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    // use messages already existing in the schema for backend initialization
    let daniella_message = "hi! this is a secret message meant only for daniella. >:(".to_string();
    let alex_message1 = "(1) i'm at work so i cant talk right now".to_string();
    let alex_message2 = "(2) what do you want kiddo".to_string();
    let barry_message = "hi barry how are you doing?".to_string();

    // make sure reciever can see in recieved
    assert!(response_contains(&client, format!("/chat/daniella"), &daniella_message, ResponsePortion::RecievedChats));
    assert!(response_contains(&client, format!("/chat/alex"), &alex_message1, ResponsePortion::RecievedChats));
    assert!(response_contains(&client, format!("/chat/alex"), &alex_message2, ResponsePortion::RecievedChats));
    assert!(response_contains(&client, format!("/chat/barry"), &barry_message, ResponsePortion::RecievedChats));
}

#[test]
fn users_can_recieve_sent_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();
    
    let users = ["alex", "barry", "cher", "charlie", "daniella", "charlie", "barry", "cher", "alex"];

    // send random messages between people and then check they work
    for s in 0..users.len()-1 {
        let sender = users[s];
        let recipient = users[s + 1];
        let message = random_string(30);

        let response = client.post(format!("/chat/{sender}/send")).header(ContentType::Form)//.header("application/x-www-form-urlencoded")
                            .body(format!("recipient={recipient}&content={}", message.replace(" ", "+"))).dispatch();
        // make sure we get redirected
        assert!(is_redirect(response.status()));

        // make sure we can see it in our sent
        assert!(response_contains(&client, format!("/chat/{sender}"), &message, ResponsePortion::SentChats));

        // and they can see it in their recieved
        assert!(response_contains(&client, format!("/chat/{recipient}"), &message, ResponsePortion::RecievedChats));
    }
}

#[test]
fn other_users_cant_see_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    // use message already existing in the schema for backend initialization
    let message = "hi! this is a secret message meant only for daniella. >:(".to_string();
    let other_users = ["ali", "barry", "cher", "charlie"];

    // make sure that other users cannot see it
    for name in other_users {
        assert!(!response_contains(&client, format!("/chat/{name}"), &message, ResponsePortion::Any));
    }
}