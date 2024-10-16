DROP DATABASE IF EXISTS sniffer3;

CREATE DATABASE sniffer3;

USE sniffer3;

CREATE TABLE users (email varchar(255), apikey varchar(255), is_admin tinyint, PRIMARY KEY (apikey));

INSERT INTO users VALUES ('kinan@bab.com', '123456789', 1);
