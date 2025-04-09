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
CREATE VIEW ml_training as '"SELECT answers.grade, answers.submitted_at, users.consent_ml as consent FROM users JOIN answers on users.email = answers.email WHERE consent = ?"';
CREATE VIEW employers_release as '"SELECT users.email, users.consent_employers as consent, AVG(answers.grade) FROM users JOIN answers on users.email = answers.email GROUP BY users.email, users.consent_employers HAVING consent = ?"';
