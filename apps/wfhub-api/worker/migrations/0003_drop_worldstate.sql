-- The game state is no longer stored (the Worker fetches it through a proxy and caches it): 0002 is not needed.
DROP TABLE IF EXISTS worldstate;
