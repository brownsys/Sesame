CREATE TABLE users (email varchar(255), apikey varchar(255), is_admin int, is_manager int, pseudonym varchar(255), gender varchar(255), age int, ethnicity varchar(255), is_remote int, education varchar(255), consent_employers int, consent_ml int, PRIMARY KEY (apikey));
CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
CREATE TABLE questions (id int AUTO_INCREMENT PRIMARY KEY, lec int REFERENCES lectures(id), q int, question text);
CREATE TABLE answers (id int AUTO_INCREMENT PRIMARY KEY, email varchar(255), lec int REFERENCES lectures(id), q int REFERENCES questions(id), answer text, submitted_at datetime, grade int);

-- For discussion leaders: a discussion leader has access to read the submitted answers.
CREATE TABLE discussion_leaders (id int AUTO_INCREMENT PRIMARY KEY, email varchar(255) REFERENCES users(email), lec int REFERENCES lectures(id));

CREATE VIEW lec_qcount as '"SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec"';

-- For non-integration (unboxed and plain Sesame), need distinct count.
CREATE VIEW agg_remote1 as '"SELECT users.is_remote, AVG(answers.grade) FROM users JOIN answers on users.email = answers.email GROUP BY users.is_remote"';
CREATE VIEW agg_remote2 as '"SELECT users.is_remote, COUNT(DISTINCT answers.email) as ucount FROM users JOIN answers on users.email = answers.email GROUP BY users.is_remote"';
CREATE VIEW agg_remote as '"SELECT * FROM agg_remote1 JOIN agg_remote2 ON agg_remote1.is_remote = agg_remote2.is_remote"';
CREATE VIEW agg_gender as '"SELECT users.gender, AVG(answers.grade) FROM users JOIN answers on users.email = answers.email GROUP BY users.gender"';

-- For non-integration (unboxed and plain Sesame), need consent.
CREATE VIEW ml_training as '"SELECT answers.grade, answers.submitted_at, users.consent_ml as consent FROM users JOIN answers on users.email = answers.email WHERE users.consent_ml = ?"';
CREATE VIEW employers_release as '"SELECT users.email, users.consent_employers as consent, AVG(answers.grade) FROM users JOIN answers on users.email = answers.email GROUP BY users.email, users.consent_employers HAVING consent = ?"';

-- Insert a single admin user.
INSERT INTO users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education, consent_employers, consent_ml) VALUES ('artem@brown.edu', 'ADMIN_API_KEY', 1, 1, 'admin_pseudonym', '', 21, '', 0, '', 0, 0);
INSERT INTO users (email, apikey, is_admin, is_manager, pseudonym, gender, age, ethnicity, is_remote, education, consent_employers, consent_ml) VALUES ('discussion_leader@brown.edu', 'DISCUSSION_LEADER_KEY', 0, 0, 'discussion_leader_pseudonym', '', 21, '', 0, '', 0, 0);

INSERT INTO discussion_leaders (id, email, lec) VALUES (0, 'discussion_leader@brown.edu', 0);
INSERT INTO discussion_leaders (id, email, lec) VALUES (1, 'discussion_leader@brown.edu', 1);
INSERT INTO discussion_leaders (id, email, lec) VALUES (2, 'discussion_leader@brown.edu', 2);
INSERT INTO discussion_leaders (id, email, lec) VALUES (3, 'discussion_leader@brown.edu', 3);
INSERT INTO discussion_leaders (id, email, lec) VALUES (4, 'discussion_leader@brown.edu', 4);
INSERT INTO discussion_leaders (id, email, lec) VALUES (5, 'discussion_leader@brown.edu', 5);
INSERT INTO discussion_leaders (id, email, lec) VALUES (6, 'discussion_leader@brown.edu', 6);
INSERT INTO discussion_leaders (id, email, lec) VALUES (7, 'discussion_leader@brown.edu', 7);
INSERT INTO discussion_leaders (id, email, lec) VALUES (8, 'discussion_leader@brown.edu', 8);
INSERT INTO discussion_leaders (id, email, lec) VALUES (9, 'discussion_leader@brown.edu', 9);
