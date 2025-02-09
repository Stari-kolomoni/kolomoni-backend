SELECT
        ws.word_id AS "word_id",
        ws.lemma AS "lemma",
        w.created_at AS "created_at",
        w.last_modified_at AS "last_modified_at",
        jsonb_agg_strict(DISTINCT meanings.data)::jsonb AS "meanings!"
    FROM kolomoni.word_slovene AS ws
    INNER JOIN kolomoni.word AS w
        ON w.id = ws.word_id
    INNER JOIN LATERAL (
        SELECT
                jsonb_build_object(
                    'word_id', wm_s.word_id,
                    'word_meaning_id', wm_s.id,
                    'created_at', wm_s.created_at,
                    'last_modified_at', wm_s.last_modified_at,
                    'disambiguation', wms.disambiguation,
                    'abbreviation', wms.abbreviation,
                    'description', wms.description,
                    'categories', jsonb_agg_strict(DISTINCT categories.category_id),
                    'translations', jsonb_agg_strict(DISTINCT translations.data)
                )::jsonb AS "data"
            FROM kolomoni.word_meaning wm_s
            INNER JOIN kolomoni.word_meaning_slovene AS wms
                ON wms.word_meaning_id = wm_s.id
            INNER JOIN LATERAL (
                SELECT
                        wmc_s.category_id AS "category_id"
                    FROM kolomoni.word_meaning_category AS wmc_s
                    WHERE wmc_s.word_meaning_id = wm_s.id
            ) AS categories ON TRUE
            INNER JOIN LATERAL (
                SELECT
                        jsonb_build_object(
                            'word', jsonb_build_object(
                                'word_id', w_e.id,
                                'created_at', w_e.created_at,
                                'last_modified_at', w_e.last_modified_at,
                                'lemma', we.lemma
                            ),
                            'word_meaning', jsonb_build_object(
                                'word_id', w_e.id,
                                'word_meaning_id', wm_e.id,
                                'created_at', wm_e.created_at,
                                'last_modified_at', wm_e.last_modified_at,
                                'description', wme.description,
                                'disambiguation', wme.disambiguation,
                                'abbreviation', wme.abbreviation
                            ),
                            'translated_at', wmt.translated_at,
                            'translated_by', wmt.translated_by
                        )::jsonb AS "data"
                    FROM kolomoni.word_meaning_translation AS wmt
                    INNER JOIN kolomoni.word_meaning AS wm_e
                        ON wm_e.id = wmt.english_word_meaning_id
                    INNER JOIN kolomoni.word_meaning_english AS wme
                        ON wme.word_meaning_id = wmt.english_word_meaning_id
                    INNER JOIN kolomoni.word AS w_e
                        ON w_e.id = wm_e.word_id
                    INNER JOIN kolomoni.word_english AS we
                        ON we.word_id = wm_e.word_id
                    WHERE wmt.slovene_word_meaning_id = wm_s.id
                    GROUP BY
                        w_e.id,
                        we.lemma,
                        wm_e.id,
                        wm_e.created_at,
                        wm_e.last_modified_at,
                        wme.description,
                        wme.disambiguation,
                        wme.abbreviation,
                        wmt.translated_at,
                        wmt.translated_by
            ) AS translations ON TRUE
            WHERE wm_s.word_id = ws.word_id
            GROUP BY
                wm_s.id,
                wm_s.created_at,
                wm_s.last_modified_at,
                wms.disambiguation,
                wms.abbreviation,
                wms.description
    ) AS meanings ON TRUE
    WHERE ws.lemma = $1
    GROUP BY
        ws.word_id,
        ws.lemma,
        w.created_at,
        w.last_modified_at
