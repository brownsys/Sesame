DROP DATABASE IF EXISTS chats;
CREATE DATABASE chats;
USE chats;

--INITIALIZING TABLE STRUCTURE:
CREATE TABLE chats(recipient varchar(255), sender varchar(255), content text, time datetime, group_chat varchar(255));
CREATE TABLE users(user_name varchar(255))
CREATE TABLE users_group(user_name varchar(255), access_code varchar(255));
CREATE TABLE group_chats(group_name varchar(255), admin_name varchar(255), access_code varchar(255));

-- CREATING SAMPLE GROUP
INSERT INTO users (user_name) VALUES ("alex")
INSERT INTO users (user_name) VALUES ("ali")
INSERT INTO users (user_name) VALUES ("barry")
INSERT INTO users (user_name) VALUES ("cher")
INSERT INTO users (user_name) VALUES ("charlie")
INSERT INTO users (user_name) VALUES ("daniella")
INSERT INTO users (user_name) VALUES ("daniella2")

INSERT INTO users_group (user_name, access_code) VALUES ("alex", 1)
INSERT INTO users_group (user_name, access_code) VALUES ("barry", 1)
INSERT INTO users_group (user_name, access_code) VALUES ("cher", 1)
INSERT INTO users_group (user_name, access_code) VALUES ("daniella", 1)
INSERT INTO users_group (user_name, access_code) VALUES ("alex", 2)
INSERT INTO users_group (user_name, access_code) VALUES ("daniella", 2)
INSERT INTO users_group (user_name, access_code) VALUES ("charlie", 2)

INSERT INTO group_chats (group_name, admin_name, access_code) VALUES ("testgroupchat", "daniella", 1)
INSERT INTO group_chats (group_name, admin_name, access_code) VALUES ("testgroupchat2", "daniella", 2)

--CREATING EXAMPLE CHATS:
INSERT INTO chats (recipient, sender, content, time) VALUES ("alex", "barry", "hi alex it's me barry", '2024-01-16 04:03:02')
INSERT INTO chats (recipient, sender, content, time) VALUES ("barry", "alex", "hi barry how are you doing?", '2024-01-16 04:07:49')
INSERT INTO chats (recipient, sender, content, time) VALUES ("charlie", "alex", "charlie charlie charlie", '2024-01-16 04:08:42')
INSERT INTO chats (recipient, sender, content, time) VALUES ("alex", "charlie", "(2) what do you want kiddo", '2024-01-16 04:11:19')
INSERT INTO chats (recipient, sender, content, time) VALUES ("alex", "charlie", "(1) i'm at work so i cant talk right now", '2024-01-16 04:11:13')

--FOR groupchat TESTS
INSERT INTO chats (recipient, sender, content, time, group_chat) VALUES (" ", "daniella", "hi everyone! welcome to the groupchat 'testgroupchat'!", '2024-01-16 04:11:13', "testgroupchat")
INSERT INTO chats (recipient, sender, content, time, group_chat) VALUES (" ", "alex", "thanks! im happy to be here", '2024-01-16 06:49:13', "testgroupchat")

--FOR "other_users_cant_see_chats" TEST
INSERT INTO chats (recipient, sender, content, time) VALUES ("daniella", "alex", "hi! this is a secret message meant only for daniella. >:(", '2024-01-16 04:07:49')