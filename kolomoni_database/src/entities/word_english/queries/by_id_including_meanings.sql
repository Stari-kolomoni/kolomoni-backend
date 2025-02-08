SELECT
        we.word_id as "word_id",
        we.lemma as "lemma",
        w.created_at as "created_at",
        w.last_modified_at as "last_modified_at",
        jsonb_agg_strict(DISTINCT meanings.data)::jsonb as "meanings!"
    FROM kolomoni.word_english as we
    INNER JOIN kolomoni.word as w
        ON we.word_id =  w.id
    INNER JOIN LATERAL (
        SELECT
                jsonb_build_object(
                    'word_id', wm_e.word_id,
                    'word_meaning_id', wm_e.id,
                    'created_at', wm_e.created_at,
                    'last_modified_at', wm_e.last_modified_at,
                    'disambiguation', wme.disambiguation,
                    'abbreviation', wme.abbreviation,
                    'description', wme.description,
                    'categories', jsonb_agg_strict(DISTINCT categories.category_id),
                    'translations', jsonb_agg_strict(DISTINCT translations.data)
                )::jsonb as "data"
            FROM kolomoni.word_meaning wm_e
            INNER JOIN kolomoni.word_meaning_english wme
                ON wme.word_meaning_id = wm_e.id
            INNER JOIN LATERAL (
                SELECT
                        wmc_e.category_id as "category_id"
                    FROM kolomoni.word_meaning_category wmc_e
                    WHERE wmc_e.word_meaning_id = wm_e.id
            ) categories ON TRUE
            INNER JOIN LATERAL (
                SELECT
                        jsonb_build_object(
                            'word', jsonb_build_object(
                                'word_id', w_s.id,
                                'created_at', w_s.created_at,
                                'last_modified_at', w_s.last_modified_at,
                                'lemma', ws.lemma
                            ),
                            'word_meaning', jsonb_build_object(
                                'word_id', w_s.id,
                                'word_meaning_id', wm_s.id,
                                'created_at', wm_s.created_at,
                                'last_modified_at', wm_s.last_modified_at,
                                'description', wms.description,
                                'disambiguation', wms.disambiguation,
                                'abbreviation', wms.abbreviation
                            ),
                            'translated_at', wmt.translated_at,
                            'translated_by', wmt.translated_by
                        )::jsonb as "data"
                    FROM kolomoni.word_meaning_translation wmt
                    INNER JOIN kolomoni.word_meaning wm_s
                        ON wm_s.id = wmt.slovene_word_meaning_id
                    INNER JOIN kolomoni.word_meaning_slovene as wms
                        ON wms.word_meaning_id = wmt.slovene_word_meaning_id
                    INNER JOIN kolomoni.word w_s
                        ON w_s.id = wm_s.word_id
                    INNER JOIN kolomoni.word_slovene ws
                        ON ws.word_id = wm_s.word_id
                    WHERE wmt.english_word_meaning_id = wm_e.id
                    GROUP BY
                        w_s.id,
                        ws.lemma,
                        wm_s.id,
                        wm_s.created_at,
                        wm_s.last_modified_at,
                        wms.description,
                        wms.disambiguation,
                        wms.abbreviation,
                        wmt.translated_at,
                        wmt.translated_by
            ) translations ON TRUE
            WHERE wm_e.word_id = we.word_id
            GROUP BY
                wm_e.id,
                wm_e.created_at,
                wm_e.last_modified_at,
                wme.disambiguation,
                wme.abbreviation,
                wme.description
    ) meanings ON TRUE
    WHERE we.word_id = $1
    GROUP BY
        we.word_id,
        we.lemma,
        w.created_at,
        w.last_modified_at
