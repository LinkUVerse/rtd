-- Copyright (c) LinkU Labs, Inc.
-- SPDX-License-Identifier: Apache-2.0

CREATE TABLE hist_display_checkpoints (
    cp_sequence_number BIGINT PRIMARY KEY CHECK (cp_sequence_number >= 0),
    checkpoint_digest BYTEA NOT NULL CHECK (octet_length(checkpoint_digest) = 32),
    previous_digest BYTEA CHECK (octet_length(previous_digest) = 32),
    event_count BIGINT NOT NULL CHECK (event_count >= 0),
    events_sha256 BYTEA NOT NULL CHECK (octet_length(events_sha256) = 32),
    CHECK ((cp_sequence_number = 0) = (previous_digest IS NULL))
);

CREATE TABLE hist_displays (
    object_type BYTEA NOT NULL,
    cp_sequence_number BIGINT NOT NULL REFERENCES hist_display_checkpoints,
    checkpoint_tx_index BIGINT NOT NULL CHECK (checkpoint_tx_index >= 0),
    event_index BIGINT NOT NULL CHECK (event_index >= 0),
    transaction_digest BYTEA NOT NULL CHECK (octet_length(transaction_digest) = 32),
    display_id BYTEA NOT NULL CHECK (octet_length(display_id) = 32),
    display_version INTEGER NOT NULL CHECK (display_version BETWEEN 0 AND 65535),
    display BYTEA NOT NULL,
    PRIMARY KEY (object_type, cp_sequence_number, checkpoint_tx_index, event_index),
    UNIQUE (cp_sequence_number, checkpoint_tx_index, event_index)
);

CREATE INDEX hist_displays_type_position
    ON hist_displays (object_type ASC, cp_sequence_number DESC, checkpoint_tx_index DESC, event_index DESC);

-- -1 means that no checkpoint, including genesis, has committed. The marker and
-- every event are committed atomically with this independent coverage boundary.
CREATE TABLE hist_display_coverage (
    id SMALLINT PRIMARY KEY CHECK (id = 1),
    genesis_digest BYTEA NOT NULL CHECK (octet_length(genesis_digest) = 32),
    source_id TEXT NOT NULL CHECK (length(source_id) BETWEEN 1 AND 256),
    generation TEXT NOT NULL CHECK (length(generation) BETWEEN 1 AND 256),
    reader_lo BIGINT NOT NULL DEFAULT 0 CHECK (reader_lo = 0),
    checkpoint_hi BIGINT NOT NULL CHECK (checkpoint_hi >= -1),
    checkpoint_digest BYTEA CHECK (octet_length(checkpoint_digest) = 32),
    writer_epoch BIGINT NOT NULL CHECK (writer_epoch > 0),
    CHECK ((checkpoint_hi = -1) = (checkpoint_digest IS NULL))
);

-- Replays may update an identical row, but cannot replace published bytes or
-- discard history. Pruning requires a separate, reviewed anchor policy.
CREATE FUNCTION rtd_hist_display_immutable() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP <> 'UPDATE' OR NEW IS DISTINCT FROM OLD THEN
        RAISE EXCEPTION 'Display history is immutable' USING ERRCODE = '55000';
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER hist_displays_immutable BEFORE UPDATE OR DELETE ON hist_displays
    FOR EACH ROW EXECUTE FUNCTION rtd_hist_display_immutable();
CREATE TRIGGER hist_displays_no_truncate BEFORE TRUNCATE ON hist_displays
    FOR EACH STATEMENT EXECUTE FUNCTION rtd_hist_display_immutable();
CREATE TRIGGER hist_display_checkpoints_immutable BEFORE UPDATE OR DELETE ON hist_display_checkpoints
    FOR EACH ROW EXECUTE FUNCTION rtd_hist_display_immutable();
CREATE TRIGGER hist_display_checkpoints_no_truncate BEFORE TRUNCATE ON hist_display_checkpoints
    FOR EACH STATEMENT EXECUTE FUNCTION rtd_hist_display_immutable();

CREATE FUNCTION rtd_hist_display_coverage_guard() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    old_hi BIGINT := -1;
    high_digest BYTEA;
    checkpoint_count BIGINT;
BEGIN
    IF TG_OP NOT IN ('INSERT', 'UPDATE') THEN
        RAISE EXCEPTION 'Display coverage cannot be removed' USING ERRCODE = '55000';
    END IF;
    IF TG_OP = 'UPDATE' THEN
        old_hi := OLD.checkpoint_hi;
        IF (NEW.id, NEW.genesis_digest, NEW.source_id, NEW.generation, NEW.reader_lo)
               IS DISTINCT FROM
           (OLD.id, OLD.genesis_digest, OLD.source_id, OLD.generation, OLD.reader_lo)
           OR NEW.checkpoint_hi < OLD.checkpoint_hi
           OR NEW.writer_epoch < OLD.writer_epoch THEN
            RAISE EXCEPTION 'Display coverage binding or monotonicity differs' USING ERRCODE = '55000';
        END IF;
    END IF;
    IF NEW.writer_epoch IS DISTINCT FROM
        NULLIF(current_setting('rtd.alt_writer_epoch', true), '')::BIGINT THEN
        RAISE EXCEPTION 'Display coverage writer epoch differs' USING ERRCODE = '55000';
    END IF;
    IF NEW.checkpoint_hi >= 0 THEN
        SELECT checkpoint_digest INTO high_digest FROM hist_display_checkpoints
            WHERE cp_sequence_number = NEW.checkpoint_hi;
        IF high_digest IS DISTINCT FROM NEW.checkpoint_digest THEN
            RAISE EXCEPTION 'Display coverage has no matching endpoint' USING ERRCODE = '55000';
        END IF;
        SELECT count(*) INTO checkpoint_count FROM hist_display_checkpoints
            WHERE cp_sequence_number > old_hi AND cp_sequence_number <= NEW.checkpoint_hi;
        IF checkpoint_count <> NEW.checkpoint_hi - old_hi THEN
            RAISE EXCEPTION 'Display coverage has a checkpoint gap' USING ERRCODE = '55000';
        END IF;
        IF EXISTS (
            SELECT 1 FROM hist_display_checkpoints c
            LEFT JOIN hist_display_checkpoints p ON p.cp_sequence_number = c.cp_sequence_number - 1
            WHERE c.cp_sequence_number > old_hi AND c.cp_sequence_number <= NEW.checkpoint_hi
              AND ((c.cp_sequence_number = 0 AND c.checkpoint_digest <> NEW.genesis_digest)
                OR (c.cp_sequence_number > 0 AND c.previous_digest IS DISTINCT FROM p.checkpoint_digest)
                OR c.event_count <> (SELECT count(*) FROM hist_displays d WHERE d.cp_sequence_number = c.cp_sequence_number))
        ) THEN
            RAISE EXCEPTION 'Display coverage predecessor or event set differs' USING ERRCODE = '55000';
        END IF;
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER hist_display_coverage_guard BEFORE INSERT OR UPDATE OR DELETE ON hist_display_coverage
    FOR EACH ROW EXECUTE FUNCTION rtd_hist_display_coverage_guard();
CREATE TRIGGER hist_display_coverage_no_truncate BEFORE TRUNCATE ON hist_display_coverage
    FOR EACH STATEMENT EXECUTE FUNCTION rtd_hist_display_coverage_guard();
