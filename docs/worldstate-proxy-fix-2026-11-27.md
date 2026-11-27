# World state stale / "WFHub API is down" — 2026-11-27

## Belirti
- `/status` sayfasında "WFHub API is down", uptime %0.24.
- `GET /api/worldstate/pc.json` 200 dönüyor ama `x-worldstate-age: ~1057000` (≈12 gün) ve `x-worldstate-stale: 1`.
- `fissures` boş → status cron'u "no fissures in the data" yazıp `down` işaretliyor.

## Sebep
Worker, oyunun sunucusuna (Cloudflare IP'lerine 403 verir) `WORLDSTATE_PROXY` secret'ındaki Webshare proxy'si üzerinden gidiyor.
`npx wrangler secret list` çıktısında bu secret **yoktu** (yalnızca `FEEDBACK_RATE_LIMIT_SECRET` ve `WORLDSTATE_PUSH_TOKEN`).
Yenileme her seferinde "the proxy is not configured" ile düşüyor, Worker D1'deki son kopyayı sunuyordu. Hata `stored_state`
içinde yutulduğu için dışarıdan görünmüyordu.

## Yapılanlar
1. Webshare dashboard → Proxy → Settings'ten kullanıcı adı / şifre okundu (Free plan aktif).
2. Proxy elle test edildi: `curl --proxy http://USER-rotate:PASS@p.webshare.io:80 https://api.warframe.com/cdn/worldState.php` → 3/3 `200`.
3. Cloudflare Worker `wfhub`'a secret eklendi: `WORLDSTATE_PROXY = http://USER-rotate:PASS@p.webshare.io:80/`
   (`wrangler secret put WORLDSTATE_PROXY`, değer stdin'den; repoya yazılmadı. Değer Webshare > Proxy > Settings'te.)
4. Yerelde (`wrangler dev --local`, `.dev.vars` gitignored) tunnel + proxy ile oyun verisi başarıyla çekildi, fissure'lar geldi.
5. `apps/wfhub-api/worker/src/worldstate.rs`: yenileme hatası artık `console_error!` ile log'lanıyor
   (`worldstate refresh failed (pc): <sebep>`). `npm run api:build` ile `worker/dist` yeniden derlendi.

## Durum / açık konular
- Secret eklendikten sonra prod hâlâ eski veri (age ~1058400) döndürüyordu; sebebi prod'da henüz görülemedi.
  Log satırı eklenen Worker **deploy edilmedi** (deploy izni verilmedi). Deploy sonrası `npx wrangler tail` ile
  `worldstate refresh failed` satırına bakılmalı.
- Olası adaylar: D1 yazma hatası, prod'da `startTls`/socket hatası, proxy'nin Cloudflare çıkışından reddi.
- Commit atılmadı (istek üzerine). Değişen dosyalar: `worker/src/worldstate.rs`, `worker/dist/*`, bu doküman.

## Kontrol
```sh
curl -sI https://wfhub.cagatayldzz.com/api/worldstate/pc.json | grep -i x-worldstate   # age küçük olmalı, stale yok
curl -s https://wfhub.cagatayldzz.com/api/status                                        # api: up
```
