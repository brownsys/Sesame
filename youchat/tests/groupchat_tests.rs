use alohomora::testing::BBoxClient;
use rocket::http::ContentType;
use youchat;
mod common;
use common::{response_contains, response_redirects, is_redirect, random_string, ResponsePortion};

// add test that bad gc doesn't internal server error and just redirects

#[test]
fn users_can_send_groupchats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    // send group chat
    let users = ["alex", "barry", "cher", "daniella"];
    for s in 0..users.len()-1 {
        let sender = users[s];
        let recipient = users[s + 1];

        let message = random_string(30);
        let response = client.post(format!("/chat/testgroupchat/{sender}/send")).header(ContentType::Form)//.header("application/x-www-form-urlencoded")
                            .body(format!("recipient={recipient}&content={}", message.replace(" ", "+"))).dispatch();
        
        // make sure we get redirected
        assert!(is_redirect(response.status()));

        // make sure we can see it in our sent
        assert!(response_contains(&client, format!("/chat/testgroupchat/{sender}"), &message, ResponsePortion::Any));
    }
}

#[test]
fn users_can_recieve_groupchats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    // use message already existing in the schema for backend initialization
    let message = "hi everyone! welcome to the groupchat 'testgroupchat'!".to_string();

    // make sure all members can see the chats
    let members = ["alex", "barry", "cher", "daniella"];
    for member in members {
        assert!(response_contains(&client, format!("/chat/testgroupchat/{member}"), &message, ResponsePortion::Any));
    }
}

#[test]
fn users_can_recieve_sent_groupchats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let members = ["alex", "barry", "cher", "daniella"];
    for s in 0..members.len()-1 {
        let sender = members[s];
        let recipient = members[s + 1];

        let message = random_string(30);
        let response = client.post(format!("/chat/testgroupchat/{sender}/send")).header(ContentType::Form)//.header("application/x-www-form-urlencoded")
                            .body(format!("recipient={recipient}&content={}", message.replace(" ", "+"))).dispatch();
        
        // make sure we get redirected
        assert!(is_redirect(response.status()));

        // make sure we can see it in our sent
        for member in members {
            assert!(response_contains(&client, format!("/chat/testgroupchat/{member}"), &message, ResponsePortion::Any));
        }
    }
}

#[test]
fn nonmembers_cant_see_groupchats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let non_members = [
        "testgroupchat/julia", "testgroupchat/robert", "testgroupchat2/samantha", "testgroupchat2/steven", 
        // ^^those not in either group
        "testgroupchat/charlie", "testgroupchat2/cher"];
        // ^^those in one but not the other

    for nm in non_members {
        // make sure we redirect when trying to connect as non-members
        assert!(response_redirects(&client, format!("/chat/{nm}"), "/login"));
    }
}

#[test]
fn invalid_group_name_redirects(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let invalid_gc = [random_string(10), random_string(10), random_string(10), random_string(10)];
    for gc in invalid_gc {
        assert!(response_redirects(&client, format!("/chat/{gc}/alex"), "/login"));
    }
}

#[test]
fn users_can_delete_their_own_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let my_chat = "thanks! im happy to be here";

    // make sure whole group can see your chat
    assert!(response_contains(&client, "/chat/testgroupchat/alex".to_string(), my_chat, ResponsePortion::Any));
    assert!(response_contains(&client, "/chat/testgroupchat/daniella".to_string(), my_chat, ResponsePortion::Any));
    
    // delete it (make sure we get redirected after)
    let response = client.post("/chat/testgroupchat/alex/1/delete").header(ContentType::Form).dispatch();
    assert!(is_redirect(response.status()));

    // make sure no one can see it anymore
    assert!(!response_contains(&client, "/chat/testgroupchat/alex".to_string(), my_chat, ResponsePortion::Any));
    assert!(!response_contains(&client, "/chat/testgroupchat/daniella".to_string(), my_chat, ResponsePortion::Any));
}

#[test]
fn admin_can_delete_any_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let chats_to_delete = ["hi everyone! welcome to the groupchat 'testgroupchat'!", "thanks! im happy to be here"];
    let admin = "daniella";

    for to_delete in chats_to_delete {
        let users = ["alex", "daniella", "cher"];

        // make sure all valid members can see the chat before
        for user in users {
            assert!(response_contains(&client, format!("/chat/testgroupchat/{user}"), to_delete, ResponsePortion::Any));
        }
        
        // try to delete it (make sure we get redirected after)
        let response = client.post(format!("/chat/testgroupchat/{admin}/0/delete")).header(ContentType::Form).dispatch();
        assert!(is_redirect(response.status()));

        // make sure nobody can see it after
        for user in users {
            assert!(!response_contains(&client, format!("/chat/testgroupchat/{user}"), to_delete, ResponsePortion::Any));
        }
    }
}

#[test]
fn nonmembers_cant_delete_any_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let chat_to_delete = "thanks! im happy to be here";
    let deleters = ["robert", "samantha", "frankie", "lia"];

    for deleter in deleters {
        let users = ["alex", "daniella", "cher"];

        // make sure all valid members can see the chat
        for user in users {
            assert!(response_contains(&client, format!("/chat/testgroupchat/{user}"), chat_to_delete, ResponsePortion::Any));
        }
        
        // try to delete it (make sure we get redirected back to login after)
        let response = client.post(format!("/chat/testgroupchat/{deleter}/1/delete")).header(ContentType::Form).dispatch();
        assert!(is_redirect(response.status()));
        assert!("/login" == response.headers().get_one("Location").unwrap().to_string());

        // make sure everyone can still see it
        for user in users {
            assert!(response_contains(&client, format!("/chat/testgroupchat/{user}"), chat_to_delete, ResponsePortion::Any));
        }
    }
}

#[test]
fn users_cant_delete_other_peoples_chats(){
    // initialize client connection
    let serv = youchat::build_server();
    let client: BBoxClient = BBoxClient::tracked(serv).unwrap();

    let my_chat = "thanks! im happy to be here";
    let deleters = ["barry", "cher"];

    for deleter in deleters {
        let users = ["alex", "daniella", deleter];

        // make sure whole group can see your chat
        for user in users {
            assert!(response_contains(&client, format!("/chat/testgroupchat/{user}"), my_chat, ResponsePortion::Any));
        }
        
        // try to delete it (make sure we get redirected after)
        let response = client.post(format!("/chat/testgroupchat/{deleter}/1/delete")).header(ContentType::Form).dispatch();
        assert!(is_redirect(response.status()));

        // make sure everyone can still see it
        for user in users {
            assert!(response_contains(&client, format!("/chat/testgroupchat/{user}"), my_chat, ResponsePortion::Any));
        }
    }
}