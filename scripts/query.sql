-- url (profile): https://bsky.app/profile/<did>
--
-- did identities:
-- did:plc:qprhbocwazxiq3laiilpxjje @ryanfrishkorn
--
-- spammy accounts:
-- did:plc:ylzc4aae2x5ipo6q6u2qf7x6 @mrbill420.bsky.social

-- sample database
attach './data/jetstream.duckdb' as js (read_only);

create temporary table posts_curated as
select * from js.posts using sample 1%;

detach js;

create temporary table reputation as
select did, count(*) as count, count(distinct text) as count_uniq, count_uniq / count as ratio
    from posts_curated
    group by did;

with top_dids as (
    select did from reputation
    order by ratio limit 100
), all_posts as (
    select * from posts_curated
    where did in (select did from top_dids)
), total_count as (
    select count(*) as count
    from all_posts
)
select did, count(*) as did_count, any_value(tc.count) as all_count, substr(any_value(text), 0, 80) as text
from all_posts, total_count as tc
group by did
order by did_count;

-- information
select 'posts_curated' as name, count(*) as count from posts_curated
union all
select 'did_uniq' as did_uniq, count(*) as count from reputation;
