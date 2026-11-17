create view posts as select cid::VARCHAR as cid, did::VARCHAR as did, feedpost::JSON as feedpost from 'data/recent.parquet';

-- aggregate posts
with top_reposts as (
    select
        count() as count,
        json_extract_string(feedpost, '$.embed.record.cid') as record
        from posts
        where json_extract_string(feedpost, '$.embed.$type') = 'app.bsky.embed.record'
        group by record
        order by count desc
), present_posts as (
    select cid,
        did,
        json_extract_string(feedpost, '$.text') as text
        from posts
        where cid in (select record from top_reposts)
)
-- select did, cid, text from present_posts limit 10;
select t.count,
    p.did,
    -- p.cid,
    p.text
    from present_posts as p
    inner join top_reposts as t
    on p.cid = t.record
    order by t.count desc
    limit 100;
