ALTER TABLE download_manifests
ADD COLUMN completed INTEGER NOT NULL DEFAULT 0;

UPDATE download_manifests
SET completed = 1
WHERE task_id IN (
    SELECT task_id
    FROM download_tasks
    WHERE json_extract(payload, '$.status') = 'completed'
);
