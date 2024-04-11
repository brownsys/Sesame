CREATE TABLE users (email varchar(255), apikey varchar(255), is_admin tinyint, is_manager tinyint, pseudonym varchar(255), gender varchar(255), age int, ethnicity varchar(255), is_remote tinyint, education varchar(255), consent tinyint, PRIMARY KEY (apikey));
CREATE TABLE lectures (id int, label varchar(255), PRIMARY KEY (id));
CREATE TABLE questions (lec int, q int, question text, PRIMARY KEY (lec, q));
CREATE TABLE answers (email varchar(255), lec int, q int, answer text, submitted_at datetime, grade int, PRIMARY KEY (email, lec, q));

-- For discussion leaders: a discussion leader has access to read the submitted answers.
CREATE TABLE discussion_leaders (id int PRIMARY KEY, email varchar(255), lec int);


CREATE VIEW lec_qcount as SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec;
CREATE VIEW agg_remote as SELECT users.is_remote, AVG(answers.grade), COUNT(DISTINCT users.email) as c FROM users JOIN answers on users.email = answers.email GROUP BY users.is_remote HAVING c > 2;
CREATE VIEW agg_gender as SELECT users.gender, AVG(answers.grade), COUNT(DISTINCT users.email) FROM users JOIN answers on users.email = answers.email GROUP BY users.gender;