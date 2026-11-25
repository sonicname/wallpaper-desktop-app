# Wallpaper image sources (Wallhaven first) and a cheap-to-extend provider layer

Evidence labels: **[OFFICIAL]** = owner's docs/terms. **[LIVE 2026-11-25]** = own unauthenticated test on that date (generic User-Agent; 12 Wallhaven API calls + 3 image-host requests. Ten API calls were SFW-only; two metadata-only calls used `purity=111` and `purity=001` to check anonymous key gating: the first returned SFW + Sketchy metadata, the second returned nothing; no images from them were downloaded). **[CODE]** = read in an open-source repo on 2026-11-25. **[COMMUNITY]** = forum/blog/third party. Items under "Inferences" are the researcher's own reasoning.

## Q1. Wallhaven API v1 in full detail

### Takeaway
Wallhaven is usable today without a key for SFW (and Sketchy): 45 calls per minute, JSON, strong resolution/ratio/colour filters, multi-value ratios including ultrawide and portrait. Three surprises for implementers: the random `seed` did **not** work in live tests (page 2 repeated an item from page 1, same seed gave totally different results), responses can take up to ~10 s, and there is no published licence for the images nor any API/commercial-use clause in the ToS (which dates from 2016).

### Cited Findings

**Endpoints [OFFICIAL]** — all from [Wallhaven API docs](https://wallhaven.cc/help/api)
- `GET /api/v1/w/{id}` (details; adds `uploader` and `tags[]`, which search results do not contain), `GET /api/v1/search`, `GET /api/v1/tag/{id}`, `GET /api/v1/settings?apikey=`, `GET /api/v1/collections` (own, needs key), `/collections/{USERNAME}` (public ones), `/collections/{USERNAME}/{ID}` (listing; "only the 'purity' filter will be available").
- The API is read-only ("basic GET requests via URL").

**Search parameters [OFFICIAL]** (`*` = default) — [Wallhaven API docs](https://wallhaven.cc/help/api)

| Param | Values |
|---|---|
| `q` | `tagname` (fuzzy), `-tag`, `+tag1 +tag2`, `+tag1 -tag2`, `@username`, `id:123` (exact tag, cannot be combined), `type:{png/jpg}`, `like:wallpaperID` (similar tags) |
| `categories` | bitmask general/anime/people, `111*` |
| `purity` | bitmask sfw/sketchy/nsfw, `100*`; "NSFW requires a valid API key" |
| `sorting` | `date_added*`, `relevance`, `random`, `views`, `favorites`, `toplist` |
| `order` | `desc*`, `asc` |
| `topRange` | `1d 3d 1w 1M* 3M 6M 1y`; sorting must be `toplist` |
| `atleast` | e.g. `1920x1080` minimum |
| `resolutions` | comma list of exact resolutions |
| `ratios` | comma list, e.g. `16x9,16x10` |
| `colors` | one of 29 fixed palette hexes (660000, 990000, cc0000, cc3333, ea4c88, 993399, 663399, 333399, 0066cc, 0099cc, 66cccc, 77cc33, 669900, 336600, 666600, 999900, cccc33, ffff00, ffcc33, ff9900, ff6600, cc6633, 996633, 663300, 000000, 999999, cccccc, ffffff, 424153) |
| `page` | 1.. ("Not actually infinite") |
| `seed` | `[a-zA-Z0-9]{6}`, "Optional seed for random results" |

- `sorting=hot` is in the website's sort menu but not in the API parameter table — [API docs page (site menu vs table)](https://wallhaven.cc/help/api).
- `ai_art_filter` is **not** in the official parameter table ([API docs](https://wallhaven.cc/help/api)); it is used by third-party clients, where `'1'` = filter AI art (default) and `'0'` = allow AI art — [RandomWallpaperGnome3 wallhaven.ts [CODE]](https://github.com/ifl0w/RandomWallpaperGnome3/blob/develop/src/adapter/wallhaven.ts). **[LIVE 2026-11-25]** `toplist&topRange=1w` with `ai_art_filter=0` vs `=1` returned the same total (467) and the same 24 items, so the test could not show an effect (either ignored or no AI art in that set). Upload rules state "No AI generated images." — [Wallhaven rules](https://wallhaven.cc/rules).
- The website's own ratio filter lists wide 16x9, 16x10; ultrawide 21x9, 32x9, 48x9; portrait 9x16, 10x16, 9x18; square 1x1, 3x2, 4x3, 5x4; plus "All Wide" and "All Portrait" — [Wallhaven site filter menu](https://wallhaven.cc/help/api).

**Response fields [OFFICIAL + LIVE]**
- Item: `id, url, short_url, views, favorites, source, purity (sfw/sketchy/nsfw), category, dimension_x, dimension_y, resolution, ratio (string, e.g. "0.56"), file_size (bytes), file_type (MIME), created_at, colors[5 hex], path, thumbs{large, original, small}` — [API docs](https://wallhaven.cc/help/api).
- Meta: `current_page, last_page, per_page, total, query (string | {id,tag} | null), seed (string | null)`; "Listings are limited to 24 results per page" — [API docs](https://wallhaven.cc/help/api). In docs `per_page` in user settings is a string ("24") while in meta it is a number, so parse leniently — [API docs](https://wallhaven.cc/help/api).
- `source` (original author's page) was an empty string in the sampled item **[LIVE 2026-11-25]** — [sample detail call](https://wallhaven.cc/api/v1/w/73em19).

**Ratios, live results [LIVE 2026-11-25]**
- `ratios=21x9,32x9,9x16` works: 12,319 results mixing 2160x3840, 2560x1080, 3440x1440, 3840x1080, 5120x1440, 5160x2160 — [request](https://wallhaven.cc/api/v1/search?sorting=random&purity=100&ratios=21x9,32x9,9x16). Matching is bucketed: 21x9 includes 2.33, 2.37 and 2.39 images, not only exact 64:27.
- `ratios=portrait` works (47,856 results with `atleast=1080x1920`) — [request](https://wallhaven.cc/api/v1/search?sorting=random&purity=100&ratios=portrait&atleast=1080x1920); `ratios=landscape` works (430,691 results) — [request](https://wallhaven.cc/api/v1/search?sorting=random&purity=100&ratios=landscape). Neither keyword is in the API table; both are undocumented.
- Caution: with `ratios=portrait&atleast=1080x1920`, returned items included 2160x2500, 1500x2183, 1438x2000. All satisfy both minimums, but ratios range 0.37-0.86, so "portrait" does not mean 9:16 (same request as above).

**Random + seed [LIVE 2026-11-25, conflicts with docs]**
- Docs: "Sorting by 'random' will produce a seed that can be passed between pages to ensure there are no repeats when getting a new page." — [API docs](https://wallhaven.cc/help/api).
- Observed: passing `seed=eM0dN4` returned a *different* seed in `meta` every time (AqpnHt, 1060Ud, ED1l24, Y4Y1ML). Repeating page 1 with the same seed shared 0 of 24 ids; repeating page 2 shared 0 of 24; first page 1 vs page 2 already had 1 duplicate — [request](https://wallhaven.cc/api/v1/search?sorting=random&purity=100&ratios=21x9,32x9,9x16&seed=eM0dN4). So anonymous seeds appear to be ignored today. Small sample, single day, no key.

**Rate limit [OFFICIAL + LIVE]**
- "API calls are currently limited to 45 per minute. If you do hit this limit, you will receive a 429 - Too many requests error." — [API docs](https://wallhaven.cc/help/api). So yes, the 45 is per minute.
- **[LIVE]** `X-RateLimit-Remaining` is also sent and counts down 44, 43, 42... then returns to 44 in a later window; no `X-RateLimit-Reset` or `Retry-After` on 200 responses — [API](https://wallhaven.cc/api/v1/search). 429 was not triggered on purpose, so whether a 429 carries `Retry-After` is unknown.
- **[LIVE]** Latency varied 0.5 s to 9.9 s for `sorting=random` queries (0.6, 3.9, 9.9, 4.5 s), so use timeouts of at least 20-30 s and prefetch — [API](https://wallhaven.cc/api/v1/search).

**Image/thumbnail hosts [LIVE 2026-11-25]**
- `th.wallhaven.cc` (thumbs) and `w.wallhaven.cc` (full) are separate hosts behind Cloudflare. No `X-RateLimit-*` headers; 200 without Referer; `Cache-Control: public, max-age=2592000` (30 days), `ETag`, `Last-Modified`, `Accept-Ranges: bytes`. A Range request on the full image returned `206`, so resumable downloads work. Full host sends `Access-Control-Allow-Origin: *`; the thumb host did not — [thumb](https://th.wallhaven.cc/lg/73/73em19.jpg), [full](https://w.wallhaven.cc/full/73/wallhaven-73em19.png).
- URL patterns: `https://w.wallhaven.cc/full/{first2}/wallhaven-{id}.{ext}`, `https://th.wallhaven.cc/{small|lg|orig}/{first2}/{id}.jpg` (thumbs are always .jpg) — [API docs](https://wallhaven.cc/help/api). (The docs' search sample shows an older `path` without `/full/`; use the returned value, never build it.)
- No hotlink policy is published in the API docs, FAQ or ToS — [API docs](https://wallhaven.cc/help/api), [FAQ](https://wallhaven.cc/faq), [ToS](https://wallhaven.cc/terms).
- FAQ, "Am I allowed to run scrapper/mass download scripts?": "We ask that you don't... we don't run any ads on this website. Because of that, we don't pay for the big boxes that can absorb huge requests." — [FAQ](https://wallhaven.cc/faq).

**API key and NSFW**
- Key comes from account settings and can be regenerated; send as `?apikey=` or `X-API-Key` header — [API docs](https://wallhaven.cc/help/api). Accounts require age over 18 — [ToS, Accounts](https://wallhaven.cc/terms).
- Unlocks: NSFW, own settings, own/private collections. "If an API key is provided, searches will be performed with that user's browsing settings and default filters" (so account defaults such as blacklists affect results) — [API docs](https://wallhaven.cc/help/api). No higher rate limit for keys is documented.
- Invalid key or NSFW detail without key: `401` — [API docs](https://wallhaven.cc/help/api).
- **[LIVE]** Without a key, `purity=111` returns 200 with only sfw + sketchy (NSFW silently dropped) — [request](https://wallhaven.cc/api/v1/search?sorting=random&purity=111&atleast=3840x2160); `purity=001` returns 200 with `total: 0`, not 401 — [request](https://wallhaven.cc/api/v1/search?sorting=random&purity=001). **Sketchy needs no key**, so an SFW-default app must explicitly pin `purity=100` and also check `item.purity == "sfw"` on every item.
- A community Android client demanded a key even for Sketchy "to avoid google rejecting the app" (about 4 years old) — [Wallhaven forum 2782 [COMMUNITY]](https://wallhaven.cc/forums/thread/2782).
- FAQ: "We don't have any official app... Be very careful of any apps that ask for your login information. Your API key should be sufficient" — [FAQ](https://wallhaven.cc/faq).

**Terms, commercial use, copyright**
- ToS last updated **3 Aug 2016**. No clause mentions the API, third-party apps, commercial use, or rate limits. Content licence is from uploader to Wallhaven; it "includes the right for us to make your Content available to other users of the Service, who may also use your Content subject to these Terms." New York law — [ToS](https://wallhaven.cc/terms).
- "All images remain property of their original owners." (footer, 2026) — [ToS page footer](https://wallhaven.cc/terms). "As Wallhaven's content is user-submitted, some of it will inevitably be the work of others", with DMCA/removal contact and uploader blacklist — [About](https://wallhaven.cc/about).
- Forum answer on commercial use of images: "Most likely not. All rights of reproduction and distribution are likely owned by the creators" (about 3 years old, no staff label) — [forum 3346 [COMMUNITY]](https://wallhaven.cc/forums/thread/3346).
- An early staff post said they would offer an API so that others could build apps; from a search summary, page not opened — [forum 18 [COMMUNITY]](https://wallhaven.cc/forums/thread/18).

**Stability**
- "While we will always aim to make sure changes to the API are communicated before hand, there may be issues that arise that will call for changes to be made without warning... This API is provided for free and as-is with no warranty." — [API docs](https://wallhaven.cc/help/api).
- One web search for 2026 outages/shutdowns found nothing about Wallhaven; only third-party directories — [example directory](https://matrix-db-api-directory.pages.dev/api/wallhaven).
- FAQ says the site is run by "just a few guys", no ads, Laravel/Elasticsearch/PHP 7.2 (FAQ text may be old) — [FAQ](https://wallhaven.cc/faq).

### Inferences
- Treat 45/min as a hard ceiling but budget about 20-30/min with a token bucket, `concurrency = 1-2`, and cooldown of 60 s + jitter on 429 if there is no `Retry-After`. A wallpaper changer needs very few calls: one random page gives 24 wallpapers.
- Because seed is unreliable, "infinite random scroll" must dedupe by `id` client-side. Simplest pattern: keep requesting `sorting=random` page 1 and drop already-seen ids.
- For ultrawide, use `ratios=21x9` (or `32x9`) + `atleast`, then pick best fit locally; do not rely on exact ratio equality.
- Map `uploader` as "uploaded by", not "author"; show the Wallhaven page link and `source` if present, plus a "Report/copyright" link. The uploader is often not the creator.
- Do not build a commercial feature on redistributing the images; let each user's device fetch directly from Wallhaven.

### Gaps
- Exact 429 body/headers, whether limit is per IP or per key, any separate Cloudflare limits on image hosts (not tested, to avoid abusing the service).
- Whether `seed` works with an API key.
- Real effect of `ai_art_filter` (undocumented; test inconclusive).
- No explicit permission or ban from Wallhaven staff on commercial third-party apps was found; asking the site owners is the only way to settle it.

## Q2. Candidate future sources: legal and technical constraints (state: late Nov 2026)

### Takeaway
Unsplash, Pexels and Pixabay all have terms that conflict with a stand-alone wallpaper app (Unsplash and Pexels name wallpaper apps explicitly), and Reddit is closing (RSS ended 13 Nov 2026; public API ends March 2027). Best next sources after Wallhaven: local folder, Wikimedia Commons, Openverse, The Met, NASA APOD (with cautions), then Bing/Spotlight as unofficial and fragile. All are free of charge.

### Cited Findings

**Unsplash: free, key needed, wallpaper apps prohibited**
- Guidelines (article dated **27 Jul 2026**): "You cannot replicate the core user experience of Unsplash (unofficial clients, wallpaper applications, etc.)." Also: "Your application's Access Key and Secret Key must remain confidential. This may require using a proxy if accessing the API client-side."; "Applications should not require users to register for a developer account with the Unsplash API to use your application. If needed, build a proxy that signs requests on behalf of your users..."; "All API uses must use the hotlinked image URLs returned by the API..."; on a download-like event "you must send a request to the download endpoint returned under the `photo.links.download_location` property"; attribution of photographer and Unsplash with `utm_source=your_app_name&utm_medium=referral`; "The API is to be used for non-automated, high-quality, and authentic experiences." — [Unsplash API Guidelines [OFFICIAL]](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines)
- The explainer gives a wallpaper app as the disallowed example: "Without the integration, the app has no content and no value to users"; email api@unsplash.com before integrating if unsure — [Guideline: Replicating Unsplash](https://help.unsplash.com/en/articles/2511257-guideline-replicating-unsplash) (quote via search extract; page text confirmed as summary).
- API Terms count setting a photo as background wallpaper as a download-type event that requires notice; "You may not share your Credentials with any third party"; Unsplash may charge "a technology fee to certain businesses" — [API terms](https://unsplash.com/api-terms) (via fetch extract; page blocked direct curl).
- Limits: demo 50 req/h, production **1000 req/h**; image requests to images.unsplash.com do not count — [Docs [OFFICIAL]](https://unsplash.com/documentation). Conflict: some third-party pages say 5000/h ([search summary of docs copies](https://pluralsight.com/guides/using-the-unsplash-api)); the docs fetched 2026-11-25 say 1000.
- Features: `/photos/random` with `count` up to 30, `orientation=landscape|portrait|squarish`, `content_filter=low|high`, dynamic resize by URL params; BlurHash; 3 weeks' notice for breaking changes — [Docs](https://unsplash.com/documentation). No minimum-resolution filter parameter found in the sections read.
- In practice: Variety ships an obfuscated (XOR) Unsplash key, self-throttles to 1 queue fill/h and 10 downloads/h, skips `plus.unsplash.com`, adds UTM params, and reports the download when the wallpaper is set — [Variety UnsplashDownloader [CODE]](https://github.com/varietywalls/variety/blob/master/variety/plugins/builtin/downloaders/UnsplashDownloader.py).

**Pexels: free, key needed, wallpaper apps prohibited**
- "You may not copy or replicate core functionality of Pexels (including making Pexels content available as a wallpaper app)."; "make sure to show a prominent link to Pexels"; "Always credit our photographers when possible."; "By default, the API is rate-limited to 200 requests per hour and 20,000 requests per month."; unlimited is free if terms are met — [Pexels API docs [OFFICIAL]](https://www.pexels.com/api/documentation/)
- Help centre: the ban "includes wallpaper and image gallery apps, even if they're for personal or educational purposes", but a platform that "primarily serves a different purpose" may let users select a background/wallpaper — [Pexels help (search extract; page returned 403 to fetch)](https://help.pexels.com/hc/en-us/articles/4405588861721-Can-I-use-the-API-as-a-wallpaper-app)
- ToS (20 Oct 2026) also bans using Content "to replicate a similar or competing service" and standalone distribution "as a print, wallpaper, poster" — [Pexels ToS](https://www.pexels.com/terms-of-service/)
- Features: `orientation=landscape|portrait|square`, `size=large|medium|small` (minimum), `color` name or hex, `per_page` max 80, `next_page` link; curated list, no random endpoint found — [docs](https://www.pexels.com/api/documentation/)

**Pixabay: free, key needed, not named as wallpaper ban but risky**
- 100 requests/60 s **per key** (not per IP), headers `X-RateLimit-Limit/Remaining/Reset`; "requests must be cached for 24 hours"; "the API is made for real human requests; do not send lots of automated queries. Systematic mass downloads are not allowed."; "permanent hotlinking of images (using Pixabay URLs in your app) is not allowed... please download them to your server first"; show users where images come from whenever results are shown — [Pixabay API docs [OFFICIAL]](https://pixabay.com/api/docs/)
- Without "full API access" approval you only get `largeImageURL` (max **1280 px**); `fullHDURL` (1920) and `imageURL` (original) need approval. `webformatURL` is valid 24 h; max 500 images per query — [docs](https://pixabay.com/api/docs/)
- Filters: `orientation=all|horizontal|vertical`, `min_width`, `min_height`, `colors` (named, multiple), `safesearch`, `category` (incl. backgrounds), `editors_choice`, `order=popular|latest`, `per_page` 3-200; no random — [docs](https://pixabay.com/api/docs/)
- ToS (20 Oct 2026): no selling or distributing Content "on a Standalone basis... as well as a print, wallpaper, poster"; not "use or compile any Content to replicate a similar or competing service" — [Pixabay ToS](https://pixabay.com/service/terms/)
- KDE ships a Pixabay key in plain text in its repo (`pixabayprovider.conf`) — [KDE [CODE]](https://github.com/KDE/kdeplasma-addons/blob/master/wallpapers/potd/plugins/providers/pixabayprovider.conf)

**Bing image of the day: no key, unofficial, third-party copyrighted**
- **[LIVE]** `https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8&mkt=en-US` returned 200; `n=20` still returned **8** items; item has `urlbase, copyright, copyrightlink, title, startdate, hsh, wp:true`; appending `_UHD.jpg` to `urlbase` returned 200 (3.57 MB) — [endpoint](https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8&mkt=en-US)
- "Use of this image is restricted to wallpaper only." appears as UI text in Microsoft's Bing Wallpaper service — [bingwallpaper.microsoft.com](https://bingwallpaper.microsoft.com/api/BWC/getHPImages); archived 2015-16 homepages say "Restricted to wallpaper use only" — [archive](https://webarchive.library.unt.edu/web/20161213023339mp_/http://www.bing.com). Microsoft adds that if licensing does not allow a download, the user is pointed to the Bing Wallpaper app — [Microsoft support](https://support.microsoft.com/en-us/bing/explore-the-homepage)
- Breakage precedent: KDE bug, Bing provider stopped working Jan 2025 after parameters changed — [KDE bug 498655](https://bugs.kde.org/show_bug.cgi?id=498655)

**Windows Spotlight: no key, reverse-engineered**
- v4 (Windows 11): `https://fd.api.iris.microsoft.com/v4/api/selection?&placement=88000820&bcnt=4&country=US&locale=en-US&fmt=json`, `bcnt` 1-4, up to 4K, landscape + portrait assets, no server scaling, no hashes; may answer "No ad available" to unusual user agents — [Spotlight Downloader SpotlightAPI.md [COMMUNITY]](https://github.com/ORelio/Spotlight-Downloader/blob/master/SpotlightAPI.md)
- **[LIVE]** The call works; JSON is double-encoded (a JSON string inside `batchrsp.items[].item`), gives `landscapeImage.asset` 3840x2160, `portraitImage.asset` 1080x1920, title, description, `copyright` (sample credits Getty Images) — [endpoint](https://fd.api.iris.microsoft.com/v4/api/selection?&placement=88000820&bcnt=1&country=US&locale=en-US&fmt=json). No public terms found for third parties.

**NASA APOD: key (free), service in transition**
- Params: `date` (from 1995-06-16), `start_date`/`end_date`, `count` (random, now **no greater than 25**); fields include `media_type` (image, **video, iframe**), `hdurl`, `copyright`/`credit` (now HTML). README "Final update 9-10-2026": backend rewired to a new WordPress endpoint, repo "will be archived and will be pointed to the new end point on **December 1, 2026**... You should not need to update your code" — [apod-api README [OFFICIAL]](https://github.com/nasa/apod-api)
- DEMO_KEY: 30/h and 50/day per IP; registered key 1,000/h — [api.nasa.gov docs](https://api.nasa.gov/assets/html/authentication.html). **[LIVE]** DEMO_KEY call returned HTTP **500** with `X-Ratelimit-Limit: 10` (differs from the docs) — [endpoint](https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY)
- Many APOD pictures are copyrighted by the named photographer (`copyright` field; sample values name individual photographers) — [README](https://github.com/nasa/apod-api)

**Wikimedia Commons: no key, free licences, strict identification**
- 2026 rate limits (requests/min): IP-only unidentified **10**; compliant User-Agent or browser **200**; authenticated new editors 200; established 2000; max 3 concurrent; respect `Retry-After`, else wait at least 5 s; "new in 2026 and are subject to experimentation and change"; User-Agent must contain contact info (email or URL) — [Wikimedia rate limits [OFFICIAL]](https://www.mediawiki.org/wiki/Wikimedia_APIs/Rate_limits)
- Thumbnails only at 20, 40, 60, 120, 250, 330, 500, 960, 1280, 1920, 3840 px; other hotlinked sizes are rejected; API-requested widths are rounded up — [Common thumbnail sizes](https://www.mediawiki.org/wiki/Common_thumbnail_sizes)
- **[LIVE]** `generator=random&grnnamespace=6` works; `generator=search` with `filetype:bitmap filew:>3839 incategory:"Featured_pictures_on_Wikimedia_Commons"` and `gsrsort=random` returns random large featured pictures; cursor in `continue`; `iiprop=extmetadata` gives `LicenseShortName, Artist (HTML), Credit, AttributionRequired`; `sha1` available — [Commons API](https://commons.wikimedia.org/w/api.php)

**Openverse: no key needed (registration raises limits)**
- **[LIVE]** Anonymous limits in headers: **20/min burst, 200/day**; `aspect_ratio=wide&size=large&license_type=commercial` accepted; fields `creator, creator_url, license, license_version, license_url, attribution` (ready-made string), `width, height, mature, unstable__sensitivity, foreign_landing_url, url, thumbnail`; `result_count` capped at 240 — [API call](https://api.openverse.org/v1/images/?q=mountain&aspect_ratio=wide&size=large&page_size=1&license_type=commercial&format=json). `/v1/rate_limit/` is limited to 1/second — [rate_limit](https://api.openverse.org/v1/rate_limit/)
- Caveat: `size=large` result was only 1792x1146, so a local resolution check is still needed (same call). Exact registered-tier numbers not found; docs name Standard/Enhanced/Exempt tiers — [Openverse docs](https://docs.openverse.org/api/reference/authentication_and_throttling.html)

**The Met: no key, CC0**
- "we do not require API users to register or obtain an API key... Please limit request rate to 80 requests per second"; data CC0, images for `isPublicDomain` works — [Met API docs [OFFICIAL]](https://metmuseum.github.io/)
- Search moved: `/v1.1/search` (offset/limit, max 500), old `/v1/search` retired 1 Oct 2026 — [Met docs, 4 Sep 2026 notice](https://metmuseum.github.io/). **[LIVE]** old one returns **410 Gone**, v1.1 returns 200 — [old](https://collectionapi.metmuseum.org/public/collection/v1/search?hasImages=true&q=landscape), [new](https://collectionapi.metmuseum.org/public/collection/v1.1/search?hasImages=true&q=landscape&limit=3). Search returns only ids, so N+1 calls are needed for details; no pixel size in the record fields listed.

**Art Institute of Chicago: no key**
- Public domain filter `query[term][is_public_domain]=true`; images via IIIF at `/full/843,/0/default.jpg`; for public domain works `1686,` is the largest listed; "Access to other image sizes is limited" — [AIC docs source [OFFICIAL]](https://github.com/art-institute-of-chicago/data-aggregator/blob/develop/docs/.sections/preface.md). So about 1686 px wide: too small for 2560/4K. Anonymous limit of 60 req/min is [COMMUNITY] (search summary) — [example](https://glama.ai/mcp/servers/@mikechao/artic-mcp/blob/4b62ff8895889292b41cc54ea0cbf3fc0c55c97a/src/utils/RateLimiter.ts)
- **[LIVE]** `api.artic.edu` returned 403 with a Cloudflare challenge to curl (not bypassed), so a native HTTP client may be blocked — [endpoint](https://api.artic.edu/api/v1/artworks/search)

**Reddit: effectively closing**
- 30 Sep 2026 announcement: no new public API requests after 31 Oct 2026; **RSS ended 13 Nov 2026** ("no replacement" for feeds outside communities you moderate); unregistered apps lose access from 12 Jan 2027; public Data API closes **March 2027** — [TechCrunch](https://techcrunch.com/2026/09/30/reddit-is-killing-rss-feeds-ending-public-api-access-because-of-ai-bots/), [TechRepublic](https://www.techrepublic.com/article/news-reddit-rss-public-api-shutdown/), [PPC Land](https://ppc.land/reddit-to-halt-public-data-api-access-in-march-2027-as-14-000-apps-register/). Unauthenticated `.json` shut around end of May/June 2026 — [DEV.to [COMMUNITY]](https://dev.to/tonywangca/why-reddit-blocked-unauthenticated-json-in-2026-and-how-to-still-get-reddit-data-58b9)
- Data API Terms (revised 20 Jul 2026): commercial use needs a separate agreement; may not "derive revenues" without written approval — [Reddit Data API Terms](https://redditinc.com/policies/data-api-terms)

**Generic JSON / RSS / local folder**
- Precedent: RandomWallpaperGnome3 (updated Oct 2026) has a user-configurable "Generic JSON" source (request URL + JSONPath for image, author, post, with prefixes and `@random`) plus local folder and static-URL sources — [repo [CODE]](https://github.com/ifl0w/RandomWallpaperGnome3/blob/develop/src/adapter/genericJson.ts). Variety has a Media RSS source — [repo tree](https://github.com/varietywalls/variety/tree/master/variety/plugins/builtin/downloaders)

**Summary matrix** (from the findings above; "Fit" is the researcher's judgement, not legal advice)

| Source | Auth | Limit | Res filter | Orientation | Random | Colour | Pagination | Attribution | Fit |
|---|---|---|---|---|---|---|---|---|---|
| Wallhaven | none (key optional) | 45/min | yes (min + exact) | ratios list | yes | 29 hex | page | none required; show link | First |
| Local folder | none | none | local | local | local | local | none | n/a | Yes |
| Wikimedia Commons | none, UA with contact | 200/min (10 if unidentified) | `filew:`/`fileh:` search | local check | yes | no | cursor (`continue`) | per-file licence | Good |
| Openverse | optional | 20/min, 200/day anon | coarse `size` | `aspect_ratio` | not verified | no | page, 240 cap | ready string | Good, low volume |
| The Met | none | 80/s | no | local | by picking ids | no | offset/limit | CC0 | Niche |
| AIC | none | about 60/min (community) | max 1686 px | local | not verified | no | page/limit | CC0/public domain only | Weak (403, small) |
| NASA APOD | key | 1000/h | no | no | `count` | no | date range | required where copyright | Cautious |
| Bing | none | not published | fixed sizes | landscape | no | no | idx 0..7 (date) | show copyright | Unofficial, fragile |
| Spotlight | none | not published | 4K | both assets | server picks | no | none | show copyright | Unofficial, fragile |
| Pixabay | key | 100/60 s per key | min w/h | yes | no | names | page, 500 cap | show source | Legal risk; 1280 px without approval |
| Pexels | key | 200/h | size classes | yes | no | yes | page/next_page | link + credit | Prohibited |
| Unsplash | key, confidential | 50/h demo, 1000/h | no | yes | yes (30) | search only | page | strict | Prohibited |
| Reddit | closing | n/a | no | no | no | no | cursor | n/a | Do not build |

### Inferences
- Unsplash's hotlink rule conflicts with a wallpaper app anyway: Windows needs a local file. Pixabay demands the opposite (no permanent hotlinking, 24 h cache). The model therefore needs a per-source "cache/hotlink policy".
- If wallpaper is only a secondary feature of a broader product, Pexels/Unsplash might be negotiable; for this app assume no and do not ship them.
- Wikimedia's UA-with-contact rule means the app must send a project URL in User-Agent; make `User-Agent` a per-provider setting owned by the host.

### Gaps
- Direct text of the Pexels help article and Unsplash API terms could not be fetched raw (403/bot challenge); quotes come from fetch/search extracts. Re-check in a browser before any legal decision.
- Full Microsoft licence text for Bing/Spotlight images not found.
- Reddit's primary posts (r/modnews, r/redditdev) not accessible; relied on press.
- Openverse random support, registered limits and terms not verified. John's Background Switcher: no source-level or 2026 information found.

## Q3. API keys in a desktop app

### Takeaway
A key shipped in a desktop binary is public. Only three honest options exist: keyless sources, the user's own key, or a proxy backend. Unsplash explicitly forbids asking users to register for developer keys and expects a proxy; Wallhaven and NASA work well with user-supplied keys.

### Cited Findings
- Unsplash: keys "must remain confidential. This may require using a proxy"; apps "should not require users to register for a developer account... build a proxy that signs requests on behalf of your users, allowing them to all share a single API key" — [Unsplash guidelines](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines)
- Wallhaven: keys are per user, from account settings, revocable; FAQ says the API key should be sufficient for apps — [API docs](https://wallhaven.cc/help/api), [FAQ](https://wallhaven.cc/faq)
- Pixabay: limit counts per key, "not with your IP address", so one embedded key is shared by all users at 100/min — [Pixabay docs](https://pixabay.com/api/docs/)
- NASA: personal key 1,000/h; DEMO_KEY 30/h — [NASA](https://api.nasa.gov/assets/html/authentication.html)
- Real projects ship extractable keys: Variety XOR-obfuscates its Unsplash key — [Variety](https://github.com/varietywalls/variety/blob/master/variety/plugins/builtin/downloaders/UnsplashDownloader.py); KDE stores its Pixabay key in plain text — [KDE](https://github.com/KDE/kdeplasma-addons/blob/master/wallpapers/potd/plugins/providers/pixabayprovider.conf)
- Variety also keeps per-source throttle values server-configurable, with compiled defaults (`max_downloads_per_hour`, `max_queue_fills_per_hour`), "to avoid overloading servers when multiple Variety users use the source simultaneously" — [ImageSource.py [CODE]](https://github.com/varietywalls/variety/blob/master/variety/plugins/downloaders/ImageSource.py)

### Inferences
- Store user keys in Windows Credential Manager/DPAPI, never in the JSON settings file or logs; send as header (`X-API-Key`), not in the URL, so keys do not leak into logs and cache keys.
- The host should inject secrets; provider code or manifests should only name a secret (`secret:wallhaven.apikey`). This keeps declarative or third-party providers from reading other keys.
- No proxy is needed for v1 if sources are Wallhaven + keyless ones. A proxy adds hosting cost, privacy duties and makes the developer the rate-limited party.

### Gaps
- No official statement found from Pexels/Pixabay on proxying keys in desktop apps.

## Q4. Provider design: how others do it, and comparison of mechanisms

### Takeaway
Recommended shape: one small compiled-in async provider interface plus a typed capability descriptor, and **one** built-in "declarative provider" that implements the interface from a JSON/TOML manifest. That makes simple REST/JSON sources a data file, while odd ones (Commons, Spotlight, local folder) stay native. Leave WASM for later, only if untrusted third-party plugins become a goal.

### Cited Findings

**How existing apps do it**
- **Variety** (Python, active; pushed 2026-11-13): `ImageSource` (identity, throttling, hooks `on_image_set_as_wallpaper` / `on_image_favorited`), `ConfigurableImageSource` (`validate(config)` returning config or error; `create_downloader(config)`; UI strings), `Downloader` with persisted `state.json`, `SimpleDownloader`/`DefaultDownloader` with queue filling — [plugins/downloaders [CODE]](https://github.com/varietywalls/variety/tree/master/variety/plugins/downloaders). Its Wallhaven source accepts a pasted wallhaven.cc search URL and rewrites it to the API URL — [WallhavenDownloader.py](https://github.com/varietywalls/variety/blob/master/variety/plugins/builtin/downloaders/WallhavenDownloader.py)
- **Muzei** (Android, active; pushed 2026-11-21): out-of-process provider apps. A provider implements `onLoadRequested(initial)` and pushes `Artwork{title, byline, attribution, token (unique id), persistentUri, webUri, metadata (opaque provider data)}`; the host controls display and re-downloads from `persistentUri`; optional `isArtworkValid`, `getCommandActions`, `openFile` — [Artwork.kt](https://github.com/muzei/muzei/blob/main/muzei-api/src/main/java/com/google/android/apps/muzei/api/provider/Artwork.kt), [MuzeiArtProvider.kt](https://github.com/muzei/muzei/blob/main/muzei-api/src/main/java/com/google/android/apps/muzei/api/provider/MuzeiArtProvider.kt)
- **Mihon (Tachiyomi)**: extensions are APKs loaded by class loader, each declares an extension lib version; the host accepts only supported ones (`SUPPORTED_LIB_VERSIONS = listOf(1.4, 1.6)`), checks signature/trust against stores, and reads an NSFW/content-warning flag — [ExtensionLoader.kt [CODE]](https://github.com/mihonapp/mihon/blob/main/app/src/main/java/eu/kanade/tachiyomi/extension/util/ExtensionLoader.kt). `CatalogueSource` has popular/latest/search with `page` and a `FilterList`; filters are a closed set of UI types: Header, Separator, Select, Text, CheckBox, TriState, Group, Sort — so the app renders any source's filters without source-specific UI — [CatalogueSource.kt](https://github.com/mihonapp/mihon/blob/main/source-api/src/main/kotlin/eu/kanade/tachiyomi/source/CatalogueSource.kt), [Filter.kt](https://github.com/mihonapp/mihon/blob/main/source-api/src/main/kotlin/eu/kanade/tachiyomi/source/model/Filter.kt)
- **KDE potd**: C++ plugin class + JSON metadata per provider; the interface returns only an image, remote URL, info URL, title, author; "picture of the day" only (identifier = name + date), no search or paging — [potdprovider.h](https://github.com/KDE/kdeplasma-addons/blob/master/wallpapers/potd/plugins/potdprovider.h)
- **wallflow** (Rust, docs.rs 0.6.1): `#[async_trait] trait WallpaperDownloader { async fn download(&self, config, query, opts) -> Result<Wallpaper>; fn source_name(&self) }` and a registry of `Arc<dyn ...>`; built-ins apod, bing, earthview, filesystem, picsum, reddit, unsplash, wallhaven — [crate source](https://docs.rs/crate/wallflow/latest/source/src/downloaders/)
- **RandomWallpaperGnome3**: base adapter with one abstract method `requestRandomImage(count)` and shared `fetchFile` — [baseAdapter.ts](https://github.com/ifl0w/RandomWallpaperGnome3/blob/develop/src/adapter/baseAdapter.ts)
- **Jackett** (strongest proof for declarative): **589** YAML definitions vs **86** C# indexer files in the repo on 2026-11-25; 16.4k stars — [repo tree [CODE]](https://github.com/Jackett/Jackett). Definition sections: `caps` (what searches are supported), `settings` (typed form fields rendered by the app: text, password, checkbox, select, multi-select), `login`, `search` (paths, inputs, headers, rows, fields), `download`, and a fixed list of filters (replace, regexp, dateparse, urlencode...) — [Definition format wiki](https://github.com/Jackett/Jackett/wiki/Definition-format). Definitions can be validated against a JSON schema — [Prowlarr/Indexers](https://github.com/Prowlarr/Indexers)

**Runtime facts**
- Wasmtime C API library on Linux x86_64 (measured 2024-12-12): 19 MB default release, 2.1 MB with `--no-default-features`, 1.2 MB with LTO/size flags, about 0.7 MB with nightly tricks; the smallest variants drop features — [Wasmtime minimal embedding](https://docs.wasmtime.dev/examples-minimal.html)
- Extism manifest has `allowed_hosts` ("if this is empty then no hosts may be accessed"), `allowed_paths`, `timeout_ms`, `memory.max_pages`, `max_http_response_bytes`, `config`, wasm `hash` — [manifest/src/lib.rs](https://github.com/extism/extism/blob/main/manifest/src/lib.rs). Latest release v1.41.0 (9 Oct 2026); Windows MSVC lib archive is 39.8 MB compressed (dev archive of libraries, not the size added to an app) — [release](https://github.com/extism/extism/releases/tag/v1.41.0)
- Rhai does not limit CPU by default; use `Engine::set_max_operations` — [Rhai book](https://rhai.rs/book/safety/max-operations.html). One community measurement: about 422 KB for a no_std size-optimised build (Rhai 1.18) — [rhai issue 791](https://github.com/rhaiscript/rhai/issues/791)

**Comparison** (effort/size ratings are the researcher's estimates unless cited above)

| Mechanism | Effort to add one source | Needs app release? | Security | Size / startup | Best for |
|---|---|---|---|---|---|
| Compiled-in trait/interface | about 100-300 lines + tests | Yes | Same trust as app | None | Odd APIs, local folder, v1 |
| Declarative manifest | about 40-100 lines of JSON/TOML | No (could be downloaded) | Best: no code; host controls hosts, rate, secrets | Tiny (JSONPath + template engine) | Most REST/JSON sources |
| Embedded script (Rhai/JS/Lua) | about 50-150 lines | No | Medium: must remove IO, cap ops/memory | About 0.4 MB to a few MB | Signing, odd transforms |
| WASM (Extism/wasmtime) | Toolchain + ABI + about 100+ lines | No | Strong sandbox + host allow-list | About 1-20 MB, compile cost (cacheable) | Untrusted third-party plugins |
| Sidecar process | Highest (IPC, packaging, updates, AV false positives) | No | Weak: full user rights | Extra process per plugin | Rarely justified |
| HTTP "addon" (remote server speaks our JSON schema) | Any language, hosted by author | No | Good: only data crosses | None | Community sources later |

### Inferences
**Capability model (so the UI adapts automatically).** Combine a fixed struct for things the engine understands with Mihon-style typed custom filters:

```rust
struct ProviderInfo {
  id, name, homepage, icon, version, api_version,      // reject unknown api_version (Mihon pattern)
  content: Vec<Rating>,                                 // [Sfw] | [Sfw, Sketchy, Nsfw(requires_key)]
  auth: Auth,        // None | OptionalKey{unlocks, help_url} | RequiredKey{help_url}
  legal: Legal,      // terms_url, license_kind, attribution: None|Show|Required, notes, stability: Official|Unofficial
  caps: Caps, rate: RatePolicy, cache: CachePolicy,
}
struct Caps {
  modes: Vec<Mode>,            // Random | Search | Latest | Popular | OfTheDay | Collection | Similar
  pagination: Pagination,      // None | Page{size,max} | Offset | Cursor | Date{min,max} | Seeded{reliable: bool}
  resolution: ResFilter,       // None | MinWxH | ExactList | Coarse(classes)
  aspect: AspectFilter,        // None | Orientation | RatioList{values, multi}
  colors: ColorFilter,         // None | Palette(list) | Named(list) | AnyHex
  custom: Vec<FilterSpec>,     // Select | MultiSelect | Toggle | Text | Sort - rendered generically
}
struct RatePolicy { max: u32, per: Duration, concurrency: u8, daily: Option<u32>, min_interval, honour_headers: bool }
struct CachePolicy { list_ttl, image_ttl, hotlink: MustHotlink|MayHotlink|NoPermanent, may_store_original: bool }
trait Provider {
  fn info(&self) -> &ProviderInfo;
  async fn list(&self, req: ListRequest, ctx: &Ctx) -> Result<Page, ProviderError>;   // the only mandatory call
  async fn resolve(&self, item: &Item, target: Size, ctx) -> Result<ImageRef>  { default: item.full }
  async fn details(&self, id, ctx) -> Result<Item>                              { default: unsupported }
  async fn on_event(&self, ev: Displayed|Downloaded|SetAsWallpaper|Favorited)   { default: no-op } // Unsplash/Variety need this
}
enum ProviderError { RateLimited{retry_after}, Auth{missing|invalid}, NotFound, Empty, Blocked, Network, BadResponse, Unsupported }
```
- The **host**, not providers, owns HTTP client, User-Agent, rate limiter per provider id, retry/backoff, disk cache, secrets, download, hash, decode and safety filter. Providers map request -> HTTP -> normalised items. This is what makes a new source cheap and gives one place to obey each source's rules.
- **Fallback rule:** every filter that a provider does not support is applied locally after listing (resolution, aspect, rating), with a bounded number of extra pages. Show it in the UI as "filtered on this device". Local checks are also needed because server filters are loose (Openverse "large" = 1792 px; Wallhaven 21x9 buckets).
- `resolve()` is needed for Met (id -> object), APOD (video entries), Spotlight (double-encoded JSON), Unsplash/IIIF (sized URLs).
- **Suggested manifest (illustrative, not an existing standard):**
```toml
api_version = 1
id = "wallhaven"; name = "Wallhaven"; hosts = ["wallhaven.cc","w.wallhaven.cc","th.wallhaven.cc"]
auth = { kind="optional", header="X-API-Key", secret="wallhaven.apikey" }
rate = { max=45, per="1m", safe=25, concurrency=2 }
[caps] modes=["random","search","toplist"]; pagination="page"; ratios=["16x9","16x10","21x9","32x9","9x16"]; resolution="min"
[list] url="https://wallhaven.cc/api/v1/search"
[list.query] q="{q?}" purity="100" sorting="{mode|map(random,date_added,toplist)}" atleast="{min_w}x{min_h}" ratios="{ratios|join(',')}" page="{page}"
[map] items="$.data[*]" total="$.meta.total" last_page="$.meta.last_page"
[map.item] id="$.id" page_url="$.url" full="$.path" thumb="$.thumbs.large" w="$.dimension_x" h="$.dimension_y" bytes="$.file_size" mime="$.file_type" colors="$.colors" rating="$.purity"
```
- Keep the expression language closed (templates, map, join, replace, JSONPath, date format). If a source needs more, write it natively; do not grow a programming language in TOML. Jackett's own ratio (589 vs 86) suggests most but not all sources fit this.
- Load order: built-in manifests compiled into the binary (so they are signed and tested), user manifests from a folder (off by default, warn user, show `hosts`), later optionally a signed remote index. A remote index also lets a broken source be fixed or disabled without an app release (Variety's remote throttle config is the precedent); Bing, Met, APOD and Reddit changes in 2025-2026 show this is a real need.
- Safety for user manifests: https only, host allow-list, block private/loopback IPs, size and time limits, no cookies, decode images in a limited way, secrets only for the manifest's own hosts.
- Testing: one shared contract test-suite run against every provider with recorded fixtures (first page, empty, 401, 429, malformed), plus an optional nightly live smoke test (1 call per source).

### Gaps
- No measured binary size/startup comparison of Rhai, rquickjs, mlua, Extism on Windows was found; figures above are partial and not like-for-like.
- Mihon trust documentation itself not read in detail; only code.

## Q5. Normalised data model, pagination, dedupe, caching, rate limiting, attribution

### Takeaway
Use one `WallpaperItem` with optional fields and an opaque per-provider blob, and one opaque cursor type so page/offset/date/continue tokens all look the same to the UI. Dedupe in three levels (provider id, canonical URL/file hash, perceptual hash).

### Cited Findings
- Pagination really differs: page 24 fixed with `last_page` ([Wallhaven](https://wallhaven.cc/help/api)); offset/limit up to 500 ([Met](https://metmuseum.github.io/)); `continue` token ([Commons live call](https://commons.wikimedia.org/w/api.php)); `next_page` URL ([Pexels](https://www.pexels.com/api/documentation/)); `idx`/`n`, max 8 per call in test ([Bing](https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8&mkt=en-US)); date range or random `count` ([APOD](https://github.com/nasa/apod-api)); result caps 500 ([Pixabay](https://pixabay.com/api/docs/)) and 240 anonymous in test ([Openverse](https://api.openverse.org/v1/images/?q=mountain&page_size=1&format=json)).
- Caching duties: Pixabay 24 h for requests and no permanent hotlink ([Pixabay](https://pixabay.com/api/docs/)); Wallhaven image hosts send 30-day `max-age` + ETag, API is `no-cache, private` ([live](https://th.wallhaven.cc/lg/73/73em19.jpg)); Bing images `max-age=691200` (8 days) in test; Commons API 1 h ([live](https://commons.wikimedia.org/w/api.php)); Unsplash requires the returned URLs ([Unsplash](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines)).
- Attribution data available: Openverse ready-made `attribution` string; Commons `extmetadata` (Artist/Credit are HTML); APOD `credit`/`copyright` now HTML ([APOD README](https://github.com/nasa/apod-api)); Bing `copyright` + `copyrightlink`; Spotlight `copyright` + `title`; Wallhaven `uploader` (detail only) + `source` (often empty in sample).
- Integrity fields: Commons `sha1`; Wallhaven `file_size` + `file_type`; Bing `hsh` (same live calls as above).
- Muzei separates `token` (unique id), `persistentUri` (re-downloadable image) and `webUri` (page) and keeps opaque `metadata` — [Artwork.kt](https://github.com/muzei/muzei/blob/main/muzei-api/src/main/java/com/google/android/apps/muzei/api/provider/Artwork.kt)

### Inferences
```rust
struct Item {
  key: (ProviderId, String),          // unique id; primary key
  page_url: Url, full: Option<ImageRef>, thumbs: Vec<ImageRef>,   // ImageRef{url,w?,h?,bytes?,mime?,expires_at?}
  width: Option<u32>, height: Option<u32>, colors: Vec<Rgb>, rating: Rating /* Unknown => treat as unsafe in SFW mode */,
  title, description, tags: Vec<String>, published_at,
  attribution: Attribution { author?, author_url?, uploader?, source_name, source_url, license_id?, license_url?, text?, required: bool, utm? },
  media: Image|Video|Other, event_hooks: opaque, raw: opaque json }
struct Page { items: Vec<Item>, next: Option<Cursor /* opaque string owned by provider */>, total: Option<u64> }
```
- Strip HTML from attribution text at mapping time (APOD, Commons) and keep the licence URL; always store attribution next to the downloaded file so it survives offline.
- Dedupe: (1) `key`; (2) canonical URL + `sha1/sha256` of downloaded bytes; (3) optional 64-bit dHash/pHash for cross-source duplicates (Openverse indexes Commons, so duplicates are certain). Keep a "seen" ring buffer per monitor for random mode, since Wallhaven seed is not dependable.
- Expiring URLs (Pixabay 24 h) mean: store id + `resolve()` again rather than persist URLs.
- Queue design: keep 10-30 ready items per (provider, filter set); refill when below a low-water mark; prefetch 1-2 full images; pause on metered networks; on failure fall back to next provider, then local cache, so the desktop never goes blank.
- Rate limiter: token bucket per provider with persistence across restarts (Openverse 200/day, Unsplash/Pexels hourly), header-driven adjustments, jitter on schedule so users do not hit at the same second (top of the hour).
- Show attribution in a tray menu/"About this picture" panel: author, licence, link, source. It satisfies Pexels/Pixabay/CC-BY style duties and costs one UI component.

### Gaps
- No source was found measuring perceptual-hash accuracy for wallpaper dedupe; threshold needs own tuning.
