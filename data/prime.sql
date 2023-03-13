-- users
INSERT INTO websubmit.users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education) VALUES ('jake@brown.edu', '2151e8c99e97b7da83c17089aa5c0b0ded35d749c1ebf2f513d07dcb2e23a1b2', 0, 0, 'UPPSZTXOYasyuhix', 'non-binary', 18, '...', 1, 'bachelors');
INSERT INTO websubmit.users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education) VALUES ('ann@brown.edu', '38f1f0106e87a77a2d5ac1a02b21fafc7f0a66dd9266bc68320260751c64f1b4', 0, 0, 'XXXOOqJM32INbxH1', 'female', 20, '...', 0, 'masters');
INSERT INTO websubmit.users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education) VALUES ('artem@brown.edu', '4424109d9f524aa1cc1fd730097f4c865feb2f827bfa8d1390af58da9dc090e3', 1, 1, 'DfsHoGKHMzmek7Hv', 'male', 19, '...', 0, 'high school');
INSERT INTO websubmit.users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education) VALUES ('sunny@brown.edu', 'b69045df9d15113dd330a3564a71dd2f3eab9a41c0a23e19a55b4f4baa628067', 0, 0, 'Rn6uH3QjLRl1JDec', 'female', 21, '...', 0, 'high school');
INSERT INTO websubmit.users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education) VALUES ('sullivan@brown.edu', '027259162a9102ae872e259a2c20d2263e024586fed8ad8ef5dba6abf4d6e6b2', 0, 1, 'dLtgv9m493hRDBID', 'non-binary', 35, '...', 0, 'PhD');
-- lectures
INSERT INTO websubmit.lectures (id, label) VALUES (1, '1');
-- questions
INSERT INTO websubmit.questions (lec, q, question) VALUES (1, 1, 'Beep.');
INSERT INTO websubmit.questions (lec, q, question) VALUES (1, 2, 'Boop.');
INSERT INTO websubmit.questions (lec, q, question) VALUES (1, 3, 'Beep.');
-- answers
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 1, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 2, 'boop', '2023-03-13 13:40:26', 5);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('ann@brown.edu', 1, 3, 'beep', '2023-03-13 13:40:26', 5);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 1, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 2, 'beep', '2023-03-09 13:54:05', 0);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('jake@brown.edu', 1, 3, 'boop', '2023-03-09 13:54:05', 0);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 1, 'boop', '2023-03-13 13:40:50', 0);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 2, 'boop', '2023-03-13 13:40:50', 5);
INSERT INTO websubmit.answers (email, lec, q, answer, submitted_at, grade) VALUES ('sunny@brown.edu', 1, 3, 'boop', '2023-03-13 13:40:50', 0);
