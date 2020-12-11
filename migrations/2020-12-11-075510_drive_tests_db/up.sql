CREATE TABLE "users" (
                         "id"	INTEGER NOT NULL UNIQUE,
                         "name"	TEXT NOT NULL,
                         "second_name"	TEXT NOT NULL,
                         "password"	TEXT NOT NULL,
                         "scores"	INTEGER NOT NULL  DEFAULT  '0',
                         PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE "tests" (
                         "id"	INTEGER NOT NULL UNIQUE,
                         "level"	INTEGER NOT NULL,
                         "description"	TEXT NOT NULL,
                         "answers"	TEXT NOT NULL,
                         "right_answer_id"	INTEGER NOT NULL,
                         PRIMARY KEY("id" AUTOINCREMENT)
);