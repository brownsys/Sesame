-- DROP DATABASE IF EXISTS alohomora;
-- CREATE DATABASE alohomora;
USE alohomora;

-- -- Schema.
-- CREATE TABLE users (email varchar(255), apikey varchar(255), is_admin tinyint, is_manager tinyint, pseudonym varchar(255), gender varchar(255), age int, ethnicity varchar(255), is_remote tinyint, education varchar(255), consent tinyint, PRIMARY KEY (apikey));
-- CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
-- CREATE TABLE questions (lec int, q int, question text, PRIMARY KEY (lec, q));
-- CREATE TABLE answers (email varchar(255), lec int, q int, answer text, submitted_at datetime, grade int, PRIMARY KEY (email, lec, q));

-- -- For discussion leaders: a discussion leader has access to read the submitted answers.
-- CREATE TABLE discussion_leaders (id int PRIMARY KEY, email varchar(255), lec int);

-- -- View for query.
-- CREATE VIEW lec_qcount as SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec;

-- discussion leaders
INSERT INTO discussion_leaders (id, email, lec) VALUES (1, 'corinn@brown.edu', 1);
INSERT INTO discussion_leaders (id, email, lec) VALUES (2, 'artem@brown.edu', 1);


-- users
INSERT INTO users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education, consent) VALUES ('jake@brown.edu', '2151e8c99e97b7da83c17089aa5c0b0ded35d749c1ebf2f513d07dcb2e23a1b2', 0, 0, 'UPPSZTXOYasyuhix', 'non-binary', 18, '...', 1, 'bachelors', 1);
INSERT INTO users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education, consent) VALUES ('ann@brown.edu', '38f1f0106e87a77a2d5ac1a02b21fafc7f0a66dd9266bc68320260751c64f1b4', 0, 0, 'XXXOOqJM32INbxH1', 'female', 20, '...', 0, 'masters', 0);
INSERT INTO users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education, consent) VALUES ('artem@brown.edu', '4424109d9f524aa1cc1fd730097f4c865feb2f827bfa8d1390af58da9dc090e3', 1, 1, 'DfsHoGKHMzmek7Hv', 'male', 19, '...', 0, 'high school', 0);
INSERT INTO users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education, consent) VALUES ('sunny@brown.edu', 'b69045df9d15113dd330a3564a71dd2f3eab9a41c0a23e19a55b4f4baa628067', 0, 0, 'Rn6uH3QjLRl1JDec', 'female', 21, '...', 0, 'high school', 1);
INSERT INTO users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education, consent) VALUES ('sullivan@brown.edu', '027259162a9102ae872e259a2c20d2263e024586fed8ad8ef5dba6abf4d6e6b2', 0, 1, 'dLtgv9m493hRDBID', 'non-binary', 35, '...', 0, 'PhD', 1);
-- lectures
INSERT INTO lectures (id, label) VALUES (1, '1');
-- questions
INSERT INTO questions (lec, q, question) VALUES (1, 1, 'Beep.');
INSERT INTO questions (lec, q, question) VALUES (1, 2, 'Boop.');
INSERT INTO questions (lec, q, question) VALUES (1, 3, 'Beep.');
-- answers
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 1, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 2, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 3, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 1, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 2, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 3, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 1, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 2, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 4, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 5, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 6, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 7, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 8, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 9, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 10, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 11, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 14, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 15, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 16, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 17, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 18, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 19, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 110, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 111, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 214, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 215, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 216, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 217, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 218, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 219, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 2110, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 2111, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 3214, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 3215, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 3216, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 3217, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 3218, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 3219, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 32110, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 32111, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 431, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 4321, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 432, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 4321, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 4321, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 432, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 41, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 9321, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 421, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 71, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 471, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 4921, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 439, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 9, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 9329, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 429, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 79, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 479, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 4929, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 4399, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 90, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 93290, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 4290, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 790, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 4790, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 49290, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 43990, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 901, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 32901, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 42901, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 7901, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 47901, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 92901, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 3990, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 9015, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 3015, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 79015, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 9015, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 92015, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 395, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 47100, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 492100, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 43900, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 900, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 932900, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 42900, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 7900, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 47900, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 492900, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 439900, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 9000, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 99000, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 429000, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 79000, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 479000, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 4929000, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 9000, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 90100, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 3290100, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 4290100, 'beep', '2023-03-13 13:40:26', 5);