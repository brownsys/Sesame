CREATE VIEW lec_qcount as '"SELECT questions.lec, COUNT(questions.q) AS qcount FROM questions GROUP BY questions.lec"';

-- For non-integration (unboxed and plain Sesame), need distinct count.
CREATE VIEW agg_remote1 as '"SELECT users.is_remote, AVG(answers.grade) FROM users JOIN answers on users.email = answers.email GROUP BY users.is_remote"';
CREATE VIEW agg_remote2 as '"SELECT users.is_remote, COUNT(DISTINCT answers.email) as ucount FROM users JOIN answers on users.email = answers.email GROUP BY users.is_remote"';
CREATE VIEW agg_remote as '"SELECT * FROM agg_remote1 JOIN agg_remote2 ON agg_remote1.is_remote = agg_remote2.is_remote"';
-- CREATE VIEW agg_gender as '"SELECT users.gender, AVG(answers.grade) FROM users JOIN answers on users.email = answers.email GROUP BY users.gender"';

-- For non-integration (unboxed and plain Sesame), need consent.
CREATE VIEW ml_training as '"SELECT answers.grade, answers.submitted_at, users.consent_ml as consent FROM users JOIN answers on users.email = answers.email WHERE users.consent_ml = ?"';
CREATE VIEW employers_release as '"SELECT users.email, users.consent_employers as consent, AVG(answers.grade) FROM users JOIN answers on users.email = answers.email GROUP BY users.email, users.consent_employers HAVING consent = ?"';