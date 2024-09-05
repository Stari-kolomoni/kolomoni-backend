-- 2024-08-20: This script initializes the first version of the Stari Kolomoni database.
--
-- # UUIDv7
-- When a column of type "uuid" appears in this schema, assume that UUIDv7 is intended by default.
-- This ensures better performance for inserts when those columns are primary keys.
--
-- # Naming scheme
-- For the constraint/index naming conventions, we use a variant of the following:
-- https://software.rcc.uchicago.edu/git/help/development/database/constraint_naming_convention.md
--
-- In our case, the underscore separator should be a double underscore ("__") instead.



----
-- Create table: permission
----
CREATE TABLE kolomoni.permission (
	id integer NOT NULL,
	key text NOT NULL,
	description_en text NOT NULL,
	description_sl text NOT NULL,
	CONSTRAINT pk__permission
	    PRIMARY KEY (id),
	CONSTRAINT unique__permission__key
	    UNIQUE (key)
);

-- We set the fillfactor to 100 because permissions will be very rarely modified.
-- Also yes, Postgres does automatically create indexes for primary keys,
-- but for clarity and for a unified naming scheme, we create an index manually instead
-- (this will be done on basically all tables).
CREATE INDEX index__permission__id
    ON kolomoni.permission (id)
    WITH (FILLFACTOR = 100);

-- We set the fillfactor to 100 because permissions will be very rarely modified.
CREATE INDEX index__permission__key
    ON kolomoni.permission (key)
    WITH (FILLFACTOR = 100);




----
-- Create table: role
----
CREATE TABLE kolomoni.role (
    id integer NOT NULL,
    key text NOT NULL,
    description_en text NOT NULL,
    description_sl text NOT NULL,
    CONSTRAINT pk__role
        PRIMARY KEY (id),
    CONSTRAINT unique__role__key
        UNIQUE (key)
);

-- We set the fillfactor to 100 because roles will be very rarely modified.
CREATE INDEX index__role__id
    ON kolomoni.role (id)
    WITH (FILLFACTOR = 100);

-- We set the fillfactor to 100 because roles will be very rarely modified.
CREATE INDEX index__role__key
    ON kolomoni.role (key)
    WITH (FILLFACTOR = 100);





----
-- Create table: role_permission
----
CREATE TABLE kolomoni.role_permission (
    role_id integer NOT NULL,
    permission_id integer NOT NULL,
    CONSTRAINT pk__role_permission PRIMARY KEY (role_id, permission_id),
    CONSTRAINT fk__role_permission__role_id__role
        FOREIGN KEY (role_id)
        REFERENCES kolomoni.role (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT fk__role_permission__permission_id__permission
        FOREIGN KEY (permission_id)
        REFERENCES kolomoni.permission (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);

-- We set the fillfactor to 100 because role-permission relationships will be very rarely modified.
CREATE INDEX index__role_permission
    ON kolomoni.role_permission (role_id, permission_id)
    WITH (FILLFACTOR = 100);




----
-- Create table: user
----
CREATE TABLE kolomoni.user (
    -- Must be UUIDv7.
    id uuid NOT NULL,
    username text NOT NULL,
    display_name text NOT NULL,
    hashed_password text NOT NULL,
    joined_at timestamp with time zone NOT NULL,
    last_modified_at timestamp with time zone NOT NULL,
    last_active_at timestamp with time zone NOT NULL,
    CONSTRAINT pk__user
        PRIMARY KEY (id),
    CONSTRAINT unique__user__username
        UNIQUE (username),
    CONSTRAINT unique__user__display_name
        UNIQUE (display_name),
    CONSTRAINT check__user__last_modified_at_ge_joined_at
        CHECK (last_modified_at >= joined_at),
    CONSTRAINT check__user__last_active_at_ge_joined_at
        CHECK (last_active_at >= joined_at)
);

CREATE INDEX index__user__id
    ON kolomoni.user (id);

CREATE INDEX index__user__username
    ON kolomoni.user (username);




----
-- Create table: user_role
----
CREATE TABLE kolomoni.user_role (
    user_id uuid NOT NULL,
    role_id integer NOT NULL,
    CONSTRAINT pk__user_role
        PRIMARY KEY (user_id, role_id),
    CONSTRAINT fk__user_role__user_id__user
        FOREIGN KEY (user_id)
        REFERENCES kolomoni.user (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT fk__user_role__role_id__role
        FOREIGN KEY (role_id)
        REFERENCES kolomoni.role (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);

CREATE INDEX index__user_role
    ON kolomoni.user_role (user_id, role_id);




----
-- Create table: word
----
CREATE TABLE kolomoni.word (
    id uuid NOT NULL,
    -- Should be a Slovene or English IETF BCP 47 language tag: "sl" or "en".
    language_code text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    last_modified_at timestamp with time zone NOT NULL,
    CONSTRAINT pk__word
        PRIMARY KEY (id),
    CONSTRAINT check__word__language_code_ietf_bcp_47_sl_or_en
        CHECK (
            (language_code = 'en') OR (language_code = 'sl')
        ),
    CONSTRAINT check__word__last_modified_at_ge_created_at
        CHECK (last_modified_at >= created_at)
);

CREATE INDEX index__word__id
    ON kolomoni.word (id);

CREATE INDEX index__word__language_code
    ON kolomoni.word USING hash (language_code);




----
-- Create table: category
----
CREATE TABLE kolomoni.category (
    id uuid NOT NULL,
    parent_category_id uuid,
    name_sl text NOT NULL,
    name_en text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    last_modified_at timestamp with time zone NOT NULL,
    CONSTRAINT pk__category
        PRIMARY KEY (id),
    CONSTRAINT unique__category__name_sl
        UNIQUE (name_sl),
    CONSTRAINT unique__category__name_en
        UNIQUE (name_en),
    CONSTRAINT fk__category__parent_category_id__category
        FOREIGN KEY (parent_category_id)
        REFERENCES kolomoni.category (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL,
    CONSTRAINT check__category__last_modified_at_ge_created_at
        CHECK (last_modified_at >= created_at)
);

CREATE INDEX index__category__id
    ON kolomoni.category (id);




----
-- Create table: word_slovene
----
CREATE TABLE kolomoni.word_slovene (
    word_id uuid NOT NULL,
    lemma text NOT NULL,
    CONSTRAINT pk__word_slovene
        PRIMARY KEY (word_id),
    CONSTRAINT fk__word_slovene__word_id__word
        FOREIGN KEY (word_id)
        REFERENCES kolomoni.word (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);

CREATE INDEX index__word_slovene__word_id
    ON kolomoni.word_slovene (word_id);

CREATE INDEX index__word_slovene__lemma
    ON kolomoni.word_slovene USING hash (lemma);




----
-- Create table: word_english
----
CREATE TABLE kolomoni.word_english (
    word_id uuid NOT NULL,
    lemma text NOT NULL,
    CONSTRAINT pk__word_english
        PRIMARY KEY (word_id),
    CONSTRAINT fk__word_english__word_id__word
        FOREIGN KEY (word_id)
        REFERENCES kolomoni.word (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);

CREATE INDEX index__word_english__word_id
    ON kolomoni.word_english (word_id);

CREATE INDEX index__word_english__lemma
    ON kolomoni.word_english USING hash (lemma);




----
-- Create table: word_meaning
----
CREATE TABLE kolomoni.word_meaning (
    id uuid NOT NULL,
    word_id uuid NOT NULL,
    CONSTRAINT pk__word_meaning
        PRIMARY KEY (id),
    CONSTRAINT fk__word_meaning__word_id__word
        FOREIGN KEY (word_id)
        REFERENCES kolomoni.word (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);

CREATE INDEX index__word_meaning__id
    ON kolomoni.word_meaning (id);

CREATE INDEX index__word_meaning__word_id
    ON kolomoni.word_meaning (word_id);




----
-- Create table: word_meaning_category
----
CREATE TABLE kolomoni.word_meaning_category (
    word_meaning_id uuid NOT NULL,
    category_id uuid NOT NULL,
    CONSTRAINT pk__word_meaning_category
        PRIMARY KEY (word_meaning_id, category_id),
    CONSTRAINT fk__word_meaning__word_meaning_id__word_meaning
        FOREIGN KEY (word_meaning_id)
        REFERENCES kolomoni.word_meaning (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT fk__word_meaning__category_id__category
        FOREIGN KEY (category_id)
        REFERENCES kolomoni.category (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);

CREATE INDEX index__word_meaning_category
    ON kolomoni.word_meaning_category (word_meaning_id, category_id);

CREATE INDEX index__word_meaning_category__word_meaning_id
    ON kolomoni.word_meaning_category (word_meaning_id);

CREATE INDEX index__word_meaning_category__category_id
    ON kolomoni.word_meaning_category (category_id);




----
-- Create table: word_slovene_meaning
----
CREATE TABLE kolomoni.word_slovene_meaning (
    word_meaning_id uuid NOT NULL,
    disambiguation text,
    abbreviation text,
    description text,
    created_at timestamp with time zone NOT NULL,
    last_modified_at timestamp with time zone NOT NULL,
    CONSTRAINT pk__word_slovene_meaning
        PRIMARY KEY (word_meaning_id),
    CONSTRAINT fk__word_slovene_meaning__word_meaning_id__word_meaning
        FOREIGN KEY (word_meaning_id)
        REFERENCES kolomoni.word_meaning (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT check__word_slovene_meaning__last_modified_at_ge_created_at
        CHECK (last_modified_at >= created_at)
);




----
-- Create table: word_english_meaning
----
CREATE TABLE kolomoni.word_english_meaning (
    word_meaning_id uuid NOT NULL,
    disambiguation text,
    abbreviation text,
    description text,
    created_at timestamp with time zone NOT NULL,
    last_modified_at timestamp with time zone NOT NULL,
    CONSTRAINT pk__word_english_meaning
        PRIMARY KEY (word_meaning_id),
    CONSTRAINT fk__word_english_meaning__word_meaning_id__word_meaning
        FOREIGN KEY (word_meaning_id)
        REFERENCES kolomoni.word_meaning (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT check__word_english_meaning__last_modified_at_ge_created_at
        CHECK (last_modified_at >= created_at)
);




----
-- Create table: word_meaning_translation
----
CREATE TABLE kolomoni.word_meaning_translation (
    slovene_word_meaning_id uuid NOT NULL,
    english_word_meaning_id uuid NOT NULL,
    translated_at timestamp with time zone NOT NULL,
    -- Can be NULL, in which case the translation should be treated as a "system-provided" translation.
    -- For example, this might be the case for the initial batch of translations that are imported from spreadsheets.
    translated_by uuid,
    CONSTRAINT pk__word_translation
        PRIMARY KEY (slovene_word_meaning_id, english_word_meaning_id),
    CONSTRAINT fk__word_translation__slovene_word_meaning_id__word_meaning
        FOREIGN KEY (slovene_word_meaning_id)
        REFERENCES kolomoni.word_meaning (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT fk__word_translation__english_word_meaning_id__word_meaning FOREIGN KEY (english_word_meaning_id)
        REFERENCES kolomoni.word_meaning (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT fk__word_translation__translated_by__user FOREIGN KEY (translated_by)
        REFERENCES kolomoni.user (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL
);

CREATE INDEX index__word_meaning_translation
    ON kolomoni.word_meaning_translation (slovene_word_meaning_id, english_word_meaning_id);

CREATE INDEX index__word_meaning_translation__slovene_word_meaning_id
    ON kolomoni.word_meaning_translation (slovene_word_meaning_id);

CREATE INDEX index__word_meaning_translation__english_word_meaning_id
    ON kolomoni.word_meaning_translation (english_word_meaning_id);



----
-- Create table: user_public_data_snapshot
----
-- CREATE TABLE kolomoni.user_public_data_snapshot (
--     id uuid NOT NULL,
--     user_id uuid NOT NULL,
--     -- username and display_name record data at snapshot-time.
--     -- This data is useful if the underlying user is removed in the future.
--     username text NOT NULL,
--     display_name text NOT NULL,
--     saved_at timestamp with time zone NOT NULL,
--     CONSTRAINT pk__user_public_data_snapshot
--         PRIMARY KEY (id)
-- );
-- 
-- CREATE INDEX index__user_public_data_snapshot
--     ON kolomoni.user_public_data_snapshot (id);


----
-- Create table: edit
----
CREATE TABLE kolomoni.edit (
    -- Edit ID (UUIDv7).
    id uuid NOT NULL,
    -- TODO Make a proper schema for the edit payload (versioned JSON, defined in Rust).
    data json NOT NULL,
    performed_at timestamp with time zone NOT NULL,
    -- Directly references the responsible user inside the database. This will become null if the user is deleted.
    author_id uuid,
    -- Records the username at the time of the edit, so it can be displayed as a fallback if the true user is deleted.
    -- See `user_public_data_snapshot`.
    -- author_snapshot_id uuid NOT NULL,
    CONSTRAINT pk__edit
        PRIMARY KEY (id),
    -- CONSTRAINT fk__edit__author_snapshot_id__user_public_data_snapshot
    --     FOREIGN KEY (author_snapshot_id)
    --     REFERENCES kolomoni.user_public_data_snapshot (id)
    --     ON UPDATE CASCADE
    --     ON DELETE NO ACTION,
    CONSTRAINT fk__edit__author__user
        FOREIGN KEY (author_id)
        REFERENCES kolomoni.user (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL
);

CREATE INDEX index__edit
    ON kolomoni.edit (id);

CREATE INDEX index__edit__performed_at
    ON kolomoni.edit (performed_at);

CREATE INDEX index__edit__author_id
    ON kolomoni.edit (author_id);
