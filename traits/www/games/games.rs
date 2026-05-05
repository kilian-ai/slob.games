use serde_json::Value;
use maud::{html, DOCTYPE, PreEscaped};

pub fn games_page(_args: &[Value]) -> Value {
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "slob.games \u{2014} Games" }
                style { (PreEscaped(CSS)) }
            }
            body {
                div.games-page {
                    div.games-section id="yourGamesSection" {
                        h2 { "Your Games" }
                        div.games-grid id="localGrid" {
                            div.loading { "Loading\u{2026}" }
                        }
                    }
                    div.games-section {
                        h2 { "Community" }
                        div.games-grid id="relayGrid" {
                            div.loading { "Loading\u{2026}" }
                        }
                    }
                }
                script { (PreEscaped(JS)) }
            }
        }
    };
    Value::String(markup.into_string())
}

const CSS: &str = r##"
.games-page{max-width:900px;margin:0 auto;padding:2rem 1.5rem}
.games-section{margin-bottom:2.5rem}
.games-section h2{font-size:0.8rem;text-transform:uppercase;letter-spacing:0.1em;color:#5a6570;font-family:'Courier New',Menlo,monospace;margin-bottom:1rem;font-weight:600}
.games-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:0.75rem}
.game-card{background:#111118;border:1px solid #1a1a2e;border-radius:10px;padding:1rem 1.1rem;cursor:pointer;transition:all 0.2s;position:relative;display:flex;flex-direction:column;gap:0.35rem}
.game-card:hover{border-color:rgba(0,224,255,0.3);transform:translateY(-1px);box-shadow:0 4px 20px rgba(0,0,0,0.3)}
.game-card .gname{font-size:0.95rem;font-weight:600;color:#e8e6e3;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.game-card .gmeta{display:flex;gap:0.4rem;align-items:center;font-size:0.7rem;color:#5a6570;flex-wrap:wrap}
.game-card .gactions{display:flex;gap:0.4rem;margin-top:0.25rem}
.badge{font-size:0.6rem;padding:1px 5px;border-radius:3px;text-transform:uppercase;letter-spacing:0.04em;font-weight:600}
.badge.local{background:rgba(0,255,136,0.08);color:#00ff88}
.badge.ext{background:rgba(0,224,255,0.08);color:#00e0ff}
.badge.relay{background:rgba(255,45,120,0.08);color:#ff2d78}
.badge.pub{background:rgba(0,255,136,0.08);color:#00ff88;cursor:pointer}
.badge.pub:hover{background:rgba(0,255,136,0.15)}
.badge.draft{background:rgba(255,102,102,0.08);color:#ff6666;cursor:pointer}
.badge.draft:hover{background:rgba(255,102,102,0.15)}
.badge.publish-dim{background:rgba(120,136,158,0.12);color:#8da0b8;cursor:pointer;opacity:.82}
.badge.publish-dim:hover{background:rgba(141,160,184,0.22);color:#b9c7d9;opacity:1}
.badge.offline{background:rgba(255,176,32,0.12);color:#ffcc66;cursor:default;opacity:.95}
.publish-all-btn{font-size:0.65rem;padding:2px 8px;border-radius:4px;border:1px solid rgba(0,255,136,0.3);background:rgba(0,255,136,0.06);color:#00ff88;cursor:pointer;font-family:inherit;letter-spacing:0.04em;text-transform:uppercase;font-weight:600;transition:all 0.15s;margin-left:0.75rem;vertical-align:middle}
.publish-all-btn:hover{background:rgba(0,255,136,0.14);border-color:rgba(0,255,136,0.6)}
.publish-all-btn:disabled{opacity:0.4;cursor:default}
#yourGamesSection h2{display:inline}
.submeta{font-size:0.62rem;color:#6f7f96;opacity:.9}
.btn-del{background:none;border:1px solid rgba(255,60,60,0.2);color:#ff4444;font-size:0.6rem;padding:1px 6px;border-radius:3px;cursor:pointer;text-transform:uppercase;letter-spacing:0.04em;font-weight:600;transition:all 0.2s}
.btn-del:hover{background:rgba(255,60,60,0.12);border-color:rgba(255,60,60,0.4)}
.btn-ren{background:none;border:1px solid rgba(0,224,255,0.2);color:#00e0ff;font-size:0.6rem;padding:1px 6px;border-radius:3px;cursor:pointer;text-transform:uppercase;letter-spacing:0.04em;font-weight:600;transition:all 0.2s}
.btn-ren:hover{background:rgba(0,224,255,0.12);border-color:rgba(0,224,255,0.4)}
.play-icon{position:absolute;top:50%;right:0.75rem;transform:translateY(-50%);width:28px;height:28px;border-radius:50%;background:rgba(0,224,255,0.08);border:1px solid rgba(0,224,255,0.15);color:#00e0ff;display:flex;align-items:center;justify-content:center;font-size:0.75rem;opacity:0;transition:opacity 0.2s}
.game-card:hover .play-icon{opacity:1}
.game-card.active-game{border-color:rgba(0,255,136,0.25)}
.game-card.active-game .gname{color:#00ff88}
.empty{color:#5a6570;font-size:0.82rem;font-style:italic;padding:0.5rem 0}
.loading{color:#5a6570;font-size:0.82rem;padding:0.5rem 0}
.status-dot{width:8px;height:8px;border-radius:50%;display:inline-block;flex-shrink:0}
.status-dot.green{background:#00ff88;box-shadow:0 0 4px rgba(0,255,136,0.4)}
.status-dot.orange{background:#ffaa00;box-shadow:0 0 4px rgba(255,170,0,0.4)}
.status-dot.red{background:#ff4444;box-shadow:0 0 4px rgba(255,68,68,0.4)}
@media(max-width:640px){.games-grid{grid-template-columns:1fr}.games-page{padding:1.5rem 1rem}}
"##;

const JS: &str = r##"
(function(){
  function esc(s){var d=document.createElement('div');d.textContent=s;return d.innerHTML}
  function fmtSize(b){return b<1024?b+'B':(b/1024).toFixed(1)+'KB'}
  var RELAY_BASES=['https://relay.slob.games/sync','https://relay.traits.build/sync'];
  var RELAY='https://relay.slob.games/sync';
  var __relayMineByGameId={};
  var __relayMineByOwnerGameId={};
  var __relayUser='';
  var __reconcileInFlight=false;
  var __relayHealth={ok:true,status:200,msg:''};
  var __fetchingHashes={};
  var __githubCatalogUrl='';

  // ── Deletion blacklist: prevent re-sync of intentionally deleted games ──
  var DELETED_KEY='traits.deleted_games';
  function _readDeletedGames(){
    try{return JSON.parse(localStorage.getItem(DELETED_KEY)||'[]')}catch(_){return[]}
  }
  function _addDeletedGame(hash,gameId,name){
    var list=_readDeletedGames();
    if(hash)list.push({hash:String(hash).trim().toLowerCase(),game_id:String(gameId||'').trim().toLowerCase(),name:String(name||''),ts:Date.now()});
    // Keep max 200 entries, expire after 30 days
    var cutoff=Date.now()-30*86400000;
    list=list.filter(function(e){return e.ts>cutoff}).slice(-200);
    localStorage.setItem(DELETED_KEY,JSON.stringify(list));
  }
  function _isDeletedGame(hash,gameId){
    var list=_readDeletedGames();
    var h=String(hash||'').trim().toLowerCase();
    var g=String(gameId||'').trim().toLowerCase();
    for(var i=0;i<list.length;i++){
      if(h&&list[i].hash&&list[i].hash===h)return true;
      if(g&&list[i].game_id&&list[i].game_id===g)return true;
    }
    return false;
  }

  function _setRelayHealth(ok,status,msg){
    __relayHealth={ok:!!ok,status:Number(status||0),msg:String(msg||'')};
  }

  function relayCandidates(){
    var out=[];
    try{
      var stored=String(localStorage.getItem('traits.relay.server')||'').trim().replace(/\/+$/,'');
      if(stored&&/^https?:\/\//i.test(stored))out.push(stored+'/sync');
    }catch(_){ }
    RELAY_BASES.forEach(function(url){if(out.indexOf(url)<0)out.push(url)});
    return out;
  }

  function rememberRelayBase(base){
    RELAY=String(base||RELAY).replace(/\/+$/,'');
    try{localStorage.setItem('traits.relay.server',RELAY.replace(/\/sync$/,''))}catch(_){ }
  }

  async function ensureRelayBase(){
    var candidates=relayCandidates();
    for(var i=0;i<candidates.length;i++){
      var base=candidates[i];
      try{
        var res=await fetch(base+'/health',{cache:'no-store'});
        var body=(await res.text()).trim().toLowerCase();
        if(res.ok&&body.indexOf('ok')>=0){
          rememberRelayBase(base);
          // Fetch GitHub catalog URL in background (best-effort)
          fetch(base+'/config/github-catalog',{cache:'no-store'}).then(function(r){
            return r.ok?r.json():null;
          }).then(function(d){
            if(d&&d.url)__githubCatalogUrl=String(d.url||'');
          }).catch(function(){});
          return true;
        }
      }catch(_){ }
    }
    return false;
  }

  async function fetchJson(path,headers){
    var h=headers||{};
    try{
      var candidates=relayCandidates();
      var lastErr='';
      for(var i=0;i<candidates.length;i++){
        var base=candidates[i];
        try{
          var res=await fetch(base+path,{headers:h,cache:'no-store'});
          rememberRelayBase(base);
          var text=await res.text();
          var data=null;
          try{data=text?JSON.parse(text):null}catch(_){data=null}
          return {ok:res.ok,status:res.status,text:text,data:data};
        }catch(e){
          lastErr=String(e&&e.message||e||'network error');
        }
      }
      return {ok:false,status:0,text:lastErr||'network error',data:null};
    }catch(e){
      return {ok:false,status:0,text:String(e&&e.message||e),data:null};
    }
  }

  function getToken(){return(localStorage.getItem('traits.secret.SLOB_USER_TOKEN')||'').trim()}
  function authHeaders(){var h={'Content-Type':'application/json'};var t=getToken();if(t)h['Authorization']='Bearer '+t;return h}
  function slugify(s){
    return String(s||'').trim().toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-+|-+$/g,'')||'untitled';
  }
  function relayGameIdForLocal(id,g){
    // Use stable IDs, never name-derived slug, so rename cannot change relay identity.
    return String((g&&(g._sync_game_id||g.game_id))||id||'untitled').trim().toLowerCase();
  }
  function relayOwnerForLocal(g){
    return String((g&&(g._sync_owner||g.owner))||__relayUser||'').trim().toLowerCase();
  }
  async function shortHash(text){
    try{
      var buf=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(String(text||'')));
      var arr=new Uint8Array(buf);var hex='';
      for(var i=0;i<arr.length;i++)hex+=arr[i].toString(16).padStart(2,'0');
      return hex.slice(0,16);
    }catch(_){return ''}
  }
  function normName(v){return String(v||'').trim().toLowerCase()}
  function normHash(v){
    var s=String(v||'').trim().toLowerCase();
    if(!s)return '';
    s=s.replace(/[^a-f0-9]/g,'');
    if(!s)return '';
    return s.slice(0,16);
  }
  function uniqueLocalId(base,col){
    var id=String(base||'untitled').trim().toLowerCase().replace(/[^a-z0-9_-]+/g,'-').replace(/^-+|-+$/g,'')||'untitled';
    if(!col.games[id])return id;
    var n=2;
    while(col.games[id+'-'+n])n++;
    return id+'-'+n;
  }

  function getGameVfsStatus(contentHash){
    var hash=normHash(contentHash);
    if(!hash)return 'red';
    if(__fetchingHashes[hash])return 'orange';
    var col=readGamesCollection();
    for(var id in (col.games||{})){
      var g=col.games[id]||{};
      var h=normHash(g._sync_hash||g.checksum||'');
      if(h&&h===hash)return 'green';
    }
    return 'red';
  }

  function updateStatusDots(){
    var dots=document.querySelectorAll('.status-dot[data-hash]');
    for(var i=0;i<dots.length;i++){
      var h=dots[i].getAttribute('data-hash');
      var s=getGameVfsStatus(h);
      dots[i].className='status-dot '+s;
    }
  }

  function readPvfsFiles(){
    try{return JSON.parse(localStorage.getItem('traits.pvfs')||'{}')}catch(_){return {}}
  }

  function normalizeResourcePath(path){
    var s=String(path||'').trim();
    if(!s)return '';
    s=s.replace(/^https?:\/\/[^\/]+/i,'');
    s=s.replace(/^\.\//,'');
    s=s.replace(/^\//,'');
    s=s.split('#')[0].split('?')[0];
    if(!s||s==='canvas/app.html'||s==='canvas/games.json')return '';
    if(s.indexOf('..')>=0)return '';
    return s;
  }

  function collectResourcesForContent(content,maxBytes){
    var files=readPvfsFiles();
    var html=String(content||'');
    var refs={};
    var total=0;
    function addRef(raw){
      var path=normalizeResourcePath(raw);
      if(!path||refs[path])return;
      var val=files[path];
      if(typeof val!=='string'||!val)return;
      if((total+val.length)>(maxBytes||2097152))return;
      refs[path]=val;
      total+=val.length;
    }
    try{
      var doc=new DOMParser().parseFromString(html,'text/html');
      ['src','href','poster'].forEach(function(attr){
        var nodes=doc.querySelectorAll('['+attr+']');
        for(var i=0;i<nodes.length;i++)addRef(nodes[i].getAttribute(attr));
      });
      var styleEls=doc.querySelectorAll('style');
      for(var si=0;si<styleEls.length;si++){
        var css=String(styleEls[si].textContent||'');
        css.replace(/url\(([^)]+)\)/g,function(_,raw){
          addRef(String(raw||'').trim().replace(/^['"]|['"]$/g,''));
          return _;
        });
      }
    }catch(_){ }
    html.replace(/url\(([^)]+)\)/g,function(_,raw){
      addRef(String(raw||'').trim().replace(/^['"]|['"]$/g,''));
      return _;
    });
    return refs;
  }

  function _isSpriteOrMediaPath(p){
    var s=String(p||'').toLowerCase();
    if(/^sprites\//.test(s))return true;
    if(/^canvas\/sprites\//.test(s))return true;
    if(/^assets\//.test(s))return true;
    if(/^images\//.test(s))return true;
    if(/^textures\//.test(s))return true;
    if(/^audio\//.test(s))return true;
    return /\.(png|jpe?g|gif|webp|svg|mp3|wav|ogg|mp4|webm|aac|flac|atlas)$/.test(s);
  }

  async function collectResourcesForContentWithIdb(content,maxBytes){
    var refs=collectResourcesForContent(content,maxBytes);
    var total=0;
    for(var k in refs){ if(typeof refs[k]==='string') total+=refs[k].length; }
    var cap=maxBytes||(8*1024*1024);
    var vfs=window.__traitsBrowserVfs;
    if(!vfs||typeof vfs.listIndexedDbEntries!=='function')return refs;
    try{
      var entries=await vfs.listIndexedDbEntries('');
      if(!Array.isArray(entries))return refs;
      for(var i=0;i<entries.length;i++){
        var e=entries[i]||{};
        var path=normalizeResourcePath(e.path||'');
        if(!path||refs[path])continue;
        if(!_isSpriteOrMediaPath(path))continue;
        var val=(typeof e.content==='string')?e.content:'';
        if(!val)continue;
        if((total+val.length)>cap)continue;
        refs[path]=val;
        total+=val.length;
      }
    }catch(_){}
    return refs;
  }

  function getSdk(){
    return window._traitsSDK||null;
  }

  function isTraitCallOk(res){
    if(!res||res.ok===false)return false;
    if(res.result&&typeof res.result==='object'&&res.result.ok===false)return false;
    return true;
  }

  async function commitRevisionSnapshot(gameKey,name,version,content,resources){
    var sdk=getSdk();
    if(!sdk)return false;
    var text=String(content||'');
    if(!text)return false;
    try{
      await sdk.call('sys.game_vcs',[
        'commit',
        String(gameKey||''),
        text,
        String(name||'untitled'),
        String(version||''),
        JSON.stringify(resources&&typeof resources==='object'?resources:{})
      ]);
      return true;
    }catch(_){return false}
  }

  async function fetchInternalGames(){
    var r=await fetchJson('/internal/games',authHeaders());
    if(!r.ok)throw new Error('Could not load private games ('+r.status+')');
    return Array.isArray(r.data)?r.data:[];
  }

  async function mergeDuplicateGamesByName(myGames){
    if(!getToken()||!__relayUser||mergeDuplicateGamesByName._busy)return false;
    mergeDuplicateGamesByName._busy=true;
    try{
      var col=readGamesCollection();
      if(!col.games)col.games={};
      var groups={};
      function pushGroup(key,row){
        if(!key)return;
        if(!groups[key])groups[key]=[];
        groups[key].push(row);
      }

      for(var id in col.games){
        var g=col.games[id]||{};
        pushGroup(slugify(g.name||id),{
          source:'local',
          id:id,
          name:String(g.name||id||'Untitled'),
          gameId:relayGameIdForLocal(id,g),
          owner:relayOwnerForLocal(g)||__relayUser||'local',
          updated:String(g.updated||g.created||''),
          version:String(g.version||''),
          content:String(g.content||''),
          resources:collectResourcesForContent(g.content||'',2*1024*1024),
          published:false,
          relayLinked:!!String(g._sync_game_id||g.game_id||'').trim(),
          active:id===col.active,
          game:g
        });
      }

      (myGames||[]).forEach(function(g){
        pushGroup(slugify(g.name||g.game_id||'untitled'),{
          source:'relay',
          id:String(g.game_id||''),
          name:String(g.name||g.game_id||'Untitled'),
          gameId:String(g.game_id||slugify(g.name||'untitled')).trim().toLowerCase(),
          owner:String(g.owner||__relayUser||'').trim().toLowerCase(),
          updated:String(g.updated||''),
          version:String(g.version||''),
          content:'',
          resources:null,
          checksum:normHash(g.checksum||g.content_hash||''),
          published:!!g.published,
          relayLinked:true,
          active:false,
          game:g
        });
      });

      var changedLocal=false;
      var changedRemote=false;
      var groupKeys=Object.keys(groups);
      for(var gi=0;gi<groupKeys.length;gi++){
        var key=groupKeys[gi];
        var variants=groups[key]||[];
        var relayCount=0;
        for(var rc=0;rc<variants.length;rc++)if(variants[rc].relayLinked)relayCount++;
        if(relayCount===0||variants.length<=1)continue;

        variants.sort(function(a,b){
          var aCanon=(a.gameId===key)?1:0;
          var bCanon=(b.gameId===key)?1:0;
          if(bCanon!==aCanon)return bCanon-aCanon;
          if(b.active!==a.active)return b.active-a.active;
          if(String(b.updated||'')!==String(a.updated||''))return String(b.updated||'').localeCompare(String(a.updated||''));
          if(b.source!==a.source)return a.source==='local'?-1:1;
          return String(a.id||'').localeCompare(String(b.id||''));
        });

        var canonical=variants[0];
        var canonicalName=canonical.name||'Untitled';
        var canonicalGameId=slugify(canonicalName||key||canonical.gameId||'untitled');
        var canonicalOwner=String(__relayUser||canonical.owner||'local').trim().toLowerCase()||'local';
        var canonicalKey=canonicalOwner+'/'+canonicalGameId;
        var seenHashes={};
        var canonicalPayload=null;
        var publishedWanted=false;

        for(var vi=0;vi<variants.length;vi++){
          var variant=variants[vi];
          var payload=null;
          if(variant.source==='local'){
            payload={
              name:variant.name,
              version:variant.version,
              content:String(variant.content||''),
              resources:variant.resources&&typeof variant.resources==='object'?variant.resources:{}
            };
          }else{
            var full=await fetchInternalGameContent(variant.gameId,variant.owner||canonicalOwner);
            if(full&&typeof full.content==='string'){
              payload={
                name:String(full.name||variant.name||'Untitled'),
                version:String(full.version||variant.version||''),
                content:String(full.content||''),
                resources:(full.resources&&typeof full.resources==='object')?full.resources:{}
              };
              variant.content=payload.content;
              variant.resources=payload.resources;
            }
          }
          if(!payload||!payload.content)continue;
          var vhash=normHash(await shortHash(payload.content));
          if(!canonicalPayload)canonicalPayload=payload;
          if(vhash&&!seenHashes[vhash]){
            seenHashes[vhash]=true;
            await commitRevisionSnapshot(canonicalKey,payload.name,payload.version,payload.content,payload.resources);
          }
          if(variant.published)publishedWanted=true;
          if(variant===canonical)canonicalPayload=payload;
        }

        if(!canonicalPayload||!canonicalPayload.content)continue;

        var canonicalLocalId='';
        for(var li=0;li<variants.length;li++){
          var localVariant=variants[li];
          if(localVariant.source!=='local')continue;
          if(localVariant.gameId===canonicalGameId||localVariant.id===canonicalGameId){
            canonicalLocalId=localVariant.id;
            break;
          }
        }
        if(!canonicalLocalId){
          for(var li2=0;li2<variants.length;li2++){
            if(variants[li2].source==='local'){canonicalLocalId=variants[li2].id;break}
          }
        }
        if(!canonicalLocalId){
          canonicalLocalId=col.games[canonicalGameId]?uniqueLocalId(canonicalGameId,col):canonicalGameId;
        }

        var canonicalGame=(col.games[canonicalLocalId]||{});
        canonicalGame.name=canonicalPayload.name||canonicalName;
        canonicalGame.content=canonicalPayload.content;
        canonicalGame.version=canonicalPayload.version||canonicalGame.version||'';
        canonicalGame.scope='internal';
        canonicalGame._scope='internal';
        canonicalGame.owner=canonicalOwner;
        canonicalGame._sync_owner=canonicalOwner;
        canonicalGame.game_id=canonicalGameId;
        canonicalGame._sync_game_id=canonicalGameId;
        canonicalGame.updated=new Date().toISOString();
        if(!canonicalGame.created)canonicalGame.created=canonicalGame.updated;
        canonicalGame._sync_hash=normHash(await shortHash(canonicalPayload.content));
        canonicalGame.checksum=canonicalGame._sync_hash;
        col.games[canonicalLocalId]=canonicalGame;
        changedLocal=true;

        for(var dl=0;dl<variants.length;dl++){
          var doomed=variants[dl];
          if(doomed.source!=='local')continue;
          if(doomed.id===canonicalLocalId)continue;
          if(col.active===doomed.id)col.active=canonicalLocalId;
          delete col.games[doomed.id];
          changedLocal=true;
        }

        try{
          var putRes=await fetch(RELAY+'/internal/game/'+encodeURIComponent(canonicalGameId),{
            method:'PUT',headers:authHeaders(),body:JSON.stringify({
              name:canonicalPayload.name||canonicalName,
              content:canonicalPayload.content,
              version:canonicalPayload.version||'',
              scope:'internal',
              resources:canonicalPayload.resources&&typeof canonicalPayload.resources==='object'?canonicalPayload.resources:{}
            })
          });
          if(putRes.ok){
            var putData=await putRes.json().catch(function(){return {}});
            canonicalGame._sync_owner=putData.owner||canonicalOwner;
            canonicalGame.owner=putData.owner||canonicalOwner;
            canonicalGame._sync_game_id=putData.game_id||canonicalGameId;
            canonicalGame.game_id=putData.game_id||canonicalGameId;
            canonicalGame._sync_hash=normHash(putData.checksum||putData.content_hash||canonicalGame._sync_hash||'');
            canonicalGame.checksum=canonicalGame._sync_hash;
            col.games[canonicalLocalId]=canonicalGame;
            changedLocal=true;
            changedRemote=true;
            if(publishedWanted){
              await fetch(RELAY+'/internal/game/'+encodeURIComponent(canonicalGameId)+'/publish',{
                method:'PATCH',headers:authHeaders(),body:JSON.stringify({published:true})
              }).catch(function(){});
            }
          }
        }catch(_){ }

        var deletedRemote={};
        for(var dr=0;dr<variants.length;dr++){
          var rv=variants[dr];
          var oldGameId='';
          if(rv.source==='relay')oldGameId=rv.gameId;
          else oldGameId=String((rv.game&&rv.game._sync_game_id)||'').trim().toLowerCase();
          if(!oldGameId||oldGameId===canonicalGameId||deletedRemote[oldGameId])continue;
          deletedRemote[oldGameId]=true;
          try{
            await fetch(RELAY+'/internal/game/'+encodeURIComponent(oldGameId)+'?owner='+encodeURIComponent(canonicalOwner),{
              method:'DELETE',headers:authHeaders()
            });
            changedRemote=true;
          }catch(_){ }
        }
      }

      if(changedLocal)writeGamesCollection(col);
      return !!(changedLocal||changedRemote);
    }finally{
      mergeDuplicateGamesByName._busy=false;
    }
  }

  function readGamesCollection(){
    try{
      var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
      var raw=pvfs['canvas/games.json'];
      if(!raw&&pvfs.files&&pvfs.files['canvas/games.json']){
        raw=String((pvfs.files['canvas/games.json']||{}).content||'');
      }
      if(raw)return JSON.parse(raw);
    }catch(e){}
    return {active:'',games:{}};
  }

  function writeGamesCollection(col){
    var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
    var json=JSON.stringify(col);
    pvfs['canvas/games.json']=json;
    if(pvfs.files&&typeof pvfs.files==='object'){
      var ts=Date.now();
      var prev=(pvfs.files['canvas/games.json']||{});
      pvfs.files['canvas/games.json']={
        content:json,
        created:(typeof prev.created==='number'?prev.created:ts),
        modified:ts
      };
    }
    localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
  }

  function setActiveGame(id){
    var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
    var raw=pvfs['canvas/games.json'];
    if(!raw&&pvfs.files&&pvfs.files['canvas/games.json'])raw=String((pvfs.files['canvas/games.json']||{}).content||'');
    var col=raw?JSON.parse(raw):{active:'',games:{}};
    col.active=id;
    if(col.games[id]&&col.games[id].content){
      pvfs['canvas/app.html']=col.games[id].content;
      if(pvfs.files&&typeof pvfs.files==='object'){
        var tsApp=Date.now();
        var prevApp=(pvfs.files['canvas/app.html']||{});
        pvfs.files['canvas/app.html']={
          content:String(col.games[id].content||''),
          created:(typeof prevApp.created==='number'?prevApp.created:tsApp),
          modified:tsApp
        };
      }
    }
    var json=JSON.stringify(col);
    pvfs['canvas/games.json']=json;
    if(pvfs.files&&typeof pvfs.files==='object'){
      var ts=Date.now();
      var prev=(pvfs.files['canvas/games.json']||{});
      pvfs.files['canvas/games.json']={
        content:json,
        created:(typeof prev.created==='number'?prev.created:ts),
        modified:ts
      };
    }
    localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
  }

  function setActiveGameFromPayload(id, g){
    var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
    var raw=pvfs['canvas/games.json'];
    if(!raw&&pvfs.files&&pvfs.files['canvas/games.json'])raw=String((pvfs.files['canvas/games.json']||{}).content||'');
    var col=raw?JSON.parse(raw):{active:'',games:{}};
    if(!col.games)col.games={};
    if(g&&typeof g==='object'){
      col.games[id]={
        name:String(g.name||'Game'),
        version:String(g.version||''),
        content:String(g.content||''),
        scope:String(g._scope||g.scope||'internal'),
        _scope:String(g._scope||g.scope||'internal'),
        owner:String(g._sync_owner||g.owner||'local'),
        _sync_owner:String(g._sync_owner||g.owner||'local'),
        game_id:String(g._sync_game_id||g.game_id||relayGameIdForLocal(id,g)||''),
        _sync_game_id:String(g._sync_game_id||g.game_id||relayGameIdForLocal(id,g)||''),
        checksum:String(g._sync_hash||g.checksum||''),
        _sync_hash:String(g._sync_hash||g.checksum||''),
        created:String(g.created||new Date().toISOString()),
        updated:new Date().toISOString()
      };
    }
    col.active=id;
    if(col.games[id]&&col.games[id].content){
      pvfs['canvas/app.html']=col.games[id].content;
      if(pvfs.files&&typeof pvfs.files==='object'){
        var tsApp=Date.now();
        var prevApp=(pvfs.files['canvas/app.html']||{});
        pvfs.files['canvas/app.html']={
          content:String(col.games[id].content||''),
          created:(typeof prevApp.created==='number'?prevApp.created:tsApp),
          modified:tsApp
        };
      }
    }
    var json=JSON.stringify(col);
    pvfs['canvas/games.json']=json;
    if(pvfs.files&&typeof pvfs.files==='object'){
      var ts=Date.now();
      var prev=(pvfs.files['canvas/games.json']||{});
      pvfs.files['canvas/games.json']={
        content:json,
        created:(typeof prev.created==='number'?prev.created:ts),
        modified:ts
      };
    }
    localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
  }

  function _compactGamesCollection(){
    // Evict duplicate and inactive games to free localStorage space.
    // Strategy: for each game name, keep only the newest entry (by id timestamp).
    // Then strip content from all non-active games that share a name with the active one.
    var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
    var raw=pvfs['canvas/games.json'];
    if(!raw&&pvfs.files&&pvfs.files['canvas/games.json'])raw=String((pvfs.files['canvas/games.json']||{}).content||'');
    if(!raw)return;
    var col=JSON.parse(raw);
    if(!col.games)return;
    // Group by name → keep newest per name
    var byName={};
    for(var gid in col.games){
      var gm=col.games[gid];
      var key=String(gm.name||'').toLowerCase().trim();
      if(!key)key=gid;
      if(!byName[key])byName[key]=[];
      byName[key].push(gid);
    }
    var removed=0;
    for(var nk in byName){
      var ids=byName[nk];
      if(ids.length<=1)continue;
      // Sort descending by id (timestamp-based) — newest first
      ids.sort(function(a,b){return b.localeCompare(a)});
      for(var i=1;i<ids.length;i++){
        if(ids[i]===col.active)continue; // never remove active
        delete col.games[ids[i]];
        removed++;
      }
    }
    if(removed>0){
      var json=JSON.stringify(col);
      pvfs['canvas/games.json']=json;
      if(pvfs.files&&typeof pvfs.files==='object'){
        var ts=Date.now();
        var prev=(pvfs.files['canvas/games.json']||{});
        pvfs.files['canvas/games.json']={content:json,created:(typeof prev.created==='number'?prev.created:ts),modified:ts};
      }
      localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
      console.log('[games] compacted: removed',removed,'duplicate game(s)');
    }
  }

  function persistCanvasLaunchPayload(id, g){
    try{
      var payload={
        id:String(id||''),
        ts:Date.now()
      };
      if(g&&typeof g==='object'){
        payload.name=String(g.name||'Game');
        payload.version=String(g.version||'');
        payload.content=String(g.content||'');
        payload.scope=String(g._scope||g.scope||'internal');
        payload.owner=String(g._sync_owner||g.owner||'local');
        payload.game_id=String(g._sync_game_id||g.game_id||relayGameIdForLocal(id,g)||'');
        payload.checksum=String(g._sync_hash||g.checksum||'');
      }
      sessionStorage.setItem('traits.canvas.launch_game',JSON.stringify(payload));
    }catch(_){ }
  }

  async function activateAndGoCanvas(id, cardGame){
    // Strategy: avoid calling sys.canvas activate/load_game from the Games page because
    // the WASM VFS is a singleton initialized once at page load, and Canvas's JS sync
    // writes directly to localStorage (flat format) which the WASM VFS never sees.
    // Instead: populate the JS pvfs + write a launch payload with full content.
    // Canvas init reads the payload and calls load_game via WASM, bridging the two stores.
    async function fetchContentForGame(g, localId){
      if(!g||typeof g!=='object')return g;
      if(typeof g.content==='string'&&g.content.length>0)return g;
      var hash=String(g._sync_hash||g.checksum||'').trim().toLowerCase();
      if(hash){
        try{
          var r=await fetch(RELAY+'/game/'+encodeURIComponent(hash));
          var d=await r.json();
          if(r.ok&&d&&typeof d.content==='string'&&d.content){
            g.content=d.content;
            if(!g.name&&d.name)g.name=d.name;
            if(!g.version&&d.version)g.version=d.version;
            return g;
          }
        }catch(_){ }
      }
      var gid=String(g._sync_game_id||g.game_id||relayGameIdForLocal(localId,g)||'').trim().toLowerCase();
      if(gid){
        try{
          var owner=String(g._sync_owner||g.owner||'').trim().toLowerCase();
          var url=RELAY+'/internal/game/'+encodeURIComponent(gid)+(owner?('?owner='+encodeURIComponent(owner)):'');
          var r2=await fetch(url,{headers:authHeaders()});
          var d2=await r2.json();
          if(r2.ok&&d2&&typeof d2.content==='string'&&d2.content){
            g.content=d2.content;
            if(!g.name&&d2.name)g.name=d2.name;
            if(!g.version&&d2.version)g.version=d2.version;
            return g;
          }
        }catch(_){ }
      }
      return g;
    }

    // 1. Resolve game object: prefer passed cardGame, else look up in local pvfs
    var g=(cardGame&&typeof cardGame==='object')?cardGame:null;
    if(!g||(typeof g.content!=='string'||!g.content)){
      var col=readGamesCollection();
      var fromPvfs=(col.games||{})[id]||null;
      if(fromPvfs)g=fromPvfs;
    }
    // 2. Hydrate content from relay if still missing
    if(g&&(typeof g.content!=='string'||!g.content)){
      try{g=await fetchContentForGame(g,id);}catch(_){}
    }
    // 3. Write game into JS pvfs so Canvas can also read it directly
    try{
      setActiveGameFromPayload(id,g);
    }catch(quotaErr){
      // localStorage full — evict duplicate/inactive games and retry
      try{
        _compactGamesCollection();
        setActiveGameFromPayload(id,g);
      }catch(_){
        // Still full — skip pvfs write; sessionStorage launch payload is the fallback
        console.warn('[games] localStorage quota exceeded, skipping pvfs write');
      }
    }
    // 4. Write launch payload to sessionStorage — Canvas init reads this and calls load_game
    persistCanvasLaunchPayload(id,g);
    goCanvas();
  }

  function goCanvas(){
    window.dispatchEvent(new CustomEvent('traits-spa-action',{detail:{spa_action:'navigate',route:'/'}}));
  }

  async function renameLocalGame(id,oldName){
    var newName=prompt('Rename game:',oldName);
    if(!newName||newName===oldName)return;
    var col=readGamesCollection();
    if(col.games&&col.games[id]){
      col.games[id].name=newName;
      col.games[id].updated=new Date().toISOString();
      writeGamesCollection(col);
      // Re-sync to relay if this game has a relay identity
      var g=col.games[id];
      var t=getToken();
      if(t&&g.content&&(g._sync_game_id||g.game_id)){
        try{await syncLocalToRelay(id)}catch(_){}
      }
    }
    await renderLocal();
    await renderRelay();
  }

  async function renameRelayGame(gameId,oldName,contentHash){
    var newName=prompt('Rename game:',oldName);
    if(!newName||newName===oldName)return;
    var t=getToken();
    if(!t){alert('Login required to rename.');return}
    try{
      // First fetch the current content from relay
      var owner='';try{var me=await fetch(RELAY+'/auth/me',{headers:authHeaders()});if(me.ok){var d=await me.json();owner=d.username||''}}catch(_){}
      var getUrl=RELAY+'/internal/game/'+encodeURIComponent(gameId)+(owner?('?owner='+encodeURIComponent(owner)):'');
      var gr=await fetch(getUrl,{headers:authHeaders()});
      if(!gr.ok){alert('Could not fetch game for rename.');return}
      var gData=await gr.json().catch(function(){return null});
      if(!gData||!gData.content){alert('Could not read game content.');return}
      // PUT back with updated name
      var payload={name:newName,content:gData.content,version:gData.version||'',scope:gData.scope||'internal'};
      var r=await fetch(RELAY+'/internal/game/'+encodeURIComponent(gameId),{
        method:'PUT',headers:Object.assign({'Content-Type':'application/json'},authHeaders()),
        body:JSON.stringify(payload)
      });
      if(!r.ok){var d2=null;try{d2=await r.json()}catch(_){} alert((d2&&d2.error)||'Rename failed');return}
    }catch(e){alert('Rename request failed');return}
    // Also rename in local collection if present
    var col=readGamesCollection();
    for(var id in (col.games||{})){
      var lg=col.games[id]||{};
      if(relayGameIdForLocal(id,lg)===String(gameId||'').toLowerCase()){
        col.games[id].name=newName;
        col.games[id].updated=new Date().toISOString();
      }
    }
    writeGamesCollection(col);
    await renderLocal();
    await renderRelay();
  }

  async function deleteLocalGame(id,name){
    if(!confirm('Delete "'+name+'"? This cannot be undone.'))return;
    var col=readGamesCollection();
    var game=col.games[id]||{};
    // Record in deletion blacklist so sync doesn't re-add
    var delHash=String(game._sync_hash||game.checksum||'').trim().toLowerCase();
    var delGameId=String(game._sync_game_id||game.game_id||'').trim().toLowerCase();
    _addDeletedGame(delHash,delGameId,name);
    delete col.games[id];
    if(col.active===id)col.active='';
    writeGamesCollection(col);
    var t=getToken();
    if(t){
      try{
        var gid=relayGameIdForLocal(id,game);
        var owner=encodeURIComponent(relayOwnerForLocal(game)||__relayUser||'');
        await fetch(RELAY+'/internal/game/'+encodeURIComponent(gid)+'?owner='+owner,{method:'DELETE',headers:authHeaders()});
      }catch(_){}
    }
    await renderLocal();
    await renderRelay();
  }

  async function setPublished(gameId,published){
    var t=getToken();
    console.log('[games:setPublished] gameId=',gameId,'published=',published,'hasToken=',!!t,'relay=',RELAY);
    if(!t){alert('Login required to publish/unpublish.');return}
    try{
      var url=RELAY+'/internal/game/'+encodeURIComponent(gameId)+'/publish';
      console.log('[games:setPublished] PATCH',url,'body=',{published:!!published});
      var r=await fetch(url,{
        method:'PATCH',headers:authHeaders(),body:JSON.stringify({published:!!published})
      });
      console.log('[games:setPublished] response status=',r.status,'ok=',r.ok);
      if(r.ok){
        var rd=null;try{rd=await r.clone().json()}catch(_){}console.log('[games:setPublished] success:',rd);
        // Mirror to GitHub when publishing (fire-and-forget)
        if(published){
          fetch(RELAY+'/internal/game/'+encodeURIComponent(gameId)+'/github-publish',{
            method:'PATCH',headers:authHeaders(),body:JSON.stringify({})
          }).then(function(gr){gr.json().then(function(gd){console.log('[games:github-publish]',gd)})}).catch(function(e){console.warn('[games:github-publish] failed:',e)});
        }
        await renderRelay();await renderLocal();
      }
      else{var d=null;try{d=await r.json()}catch(_){} console.error('[games:setPublished] FAILED:',r.status,d);alert((d&&d.error)||'Publish update failed')}
    }catch(e){console.error('[games:setPublished] exception:',e);alert('Publish request failed')}
  }

  async function syncLocalToRelay(localId){
    var t=getToken();
    if(!t){alert('Login required to publish.');return null}
    var col=readGamesCollection();
    var g=col.games[localId];
    if(!g||!g.content){alert('Nothing to publish for this game.');return null}
    // Filename (game_id) is the stable identity; mirror updates overwrite that key.
    var gameId=relayGameIdForLocal(localId,g);
    var payload={
      name:g.name||'Untitled',
      content:String(g.content||''),
      version:g.version||'',
      scope:'internal',
      resources:await collectResourcesForContentWithIdb(g.content||'',8*1024*1024)
    };
    var r=await fetch(RELAY+'/internal/game/'+encodeURIComponent(gameId),{
      method:'PUT',headers:authHeaders(),body:JSON.stringify(payload)
    });
    if(!r.ok){
      var d=null;try{d=await r.json()}catch(_){}
      throw new Error((d&&d.error)||'Sync failed');
    }
    var data=await r.json().catch(function(){return {}});
    var fresh=readGamesCollection();
    var ag=fresh.games[localId]||g;
    ag._sync_owner=data.owner||ag._sync_owner||__relayUser||'';
    ag._sync_game_id=data.game_id||gameId;
    ag._sync_hash=data.content_hash||data.checksum||ag._sync_hash||'';
    ag.checksum=data.checksum||data.content_hash||ag.checksum||'';
    ag.scope='internal';
    ag._scope='internal';
    fresh.games[localId]=ag;
    writeGamesCollection(fresh);
    var canonicalGameId=String(data.game_id||ag._sync_game_id||gameId||'').trim().toLowerCase();
    var canonicalOwner=String(data.owner||ag._sync_owner||__relayUser||'').trim().toLowerCase();
    if(canonicalGameId){
      __relayMineByGameId[canonicalGameId]={
        owner:canonicalOwner,
        game_id:canonicalGameId,
        name:ag.name||g.name||'Untitled',
        checksum:ag.checksum||ag._sync_hash||'',
        content_hash:ag._sync_hash||ag.checksum||'',
        published:false,
        scope:'internal',
        version:ag.version||g.version||'',
        size:String(ag.content||g.content||'').length,
        updated:ag.updated||new Date().toISOString()
      };
      if(canonicalOwner){
        __relayMineByOwnerGameId[canonicalOwner+'/'+canonicalGameId]=__relayMineByGameId[canonicalGameId];
      }
    }
    return {gameId:canonicalGameId||gameId,data:data};
  }

  async function publishLocalGame(localId){
    try{
      var synced=await syncLocalToRelay(localId);
      if(!synced)return;
      await setPublished(synced.gameId,true);
    }catch(e){
      alert((e&&e.message)||'Publish failed');
    }
  }

  async function fetchInternalGameContent(gameId, owner){
    try{
      var q='?owner='+encodeURIComponent(String(owner||''));
      var r=await fetch(RELAY+'/internal/game/'+encodeURIComponent(String(gameId||''))+q,{headers:authHeaders()});
      if(!r.ok)return null;
      var d=await r.json().catch(function(){return null});
      return (d&&typeof d.content==='string')?d:null;
    }catch(_){return null}
  }

  async function reconcileLocalAndRelay(myGames){
    if(__reconcileInFlight)return;
    __reconcileInFlight=true;
    try{
      var col=readGamesCollection();
      if(!col.games)col.games={};
      var changed=false;
      var didLocalPush=false;
      var mirrorKeys={};

      var localRows=[];
      for(var id in col.games){
        var lg=col.games[id]||{};
        var content=String(lg.content||'');
        localRows.push({
          id:id,
          name:normName(lg.name||id),
          size:content.length,
          hash:normHash(await shortHash(content)),
          game:lg
        });
      }

      for(var i=0;i<(myGames||[]).length;i++){
        var rg=myGames[i]||{};
        var rName=normName(rg.name||'untitled');
        var rSize=Number(rg.size||0);
        var rHash=normHash(rg.checksum||rg.content_hash||'');
        var rOwner=String(rg.owner||__relayUser||'').trim().toLowerCase();
        var rGameId=String(rg.game_id||slugify(rg.name||'untitled')).trim().toLowerCase();
        mirrorKeys[rOwner+'/'+rGameId]=true;

        var match=null;
        for(var li0=0;li0<localRows.length;li0++){
          var row0=localRows[li0];
          var g0=row0.game||{};
          if(String(g0._sync_game_id||'').trim().toLowerCase()===rGameId){
            match=row0;
            break;
          }
        }
        for(var li=0;li<localRows.length;li++){
          if(match)break;
          var row=localRows[li];
          if(row.name!==rName)continue;
          if(row.size!==rSize)continue;
          if(!row.hash||!rHash||row.hash!==rHash)continue;
          match=row;
          break;
        }

        if(match){
          var mg=col.games[match.id]||{};
          if(mg._sync_game_id!==rGameId||mg._sync_owner!==rOwner||normHash(mg._sync_hash)!==rHash){
            mg._sync_game_id=rGameId;
            mg._sync_owner=rOwner;
            mg._sync_hash=rHash||mg._sync_hash||'';
            mg.checksum=rHash||mg.checksum||'';
            if(!mg.scope)mg.scope='internal';
            col.games[match.id]=mg;
            changed=true;
          }
          continue;
        }

        var foundBySync=false;
        for(var exId in col.games){
          var ex=col.games[exId]||{};
          if(String(ex._sync_game_id||'').trim().toLowerCase()===rGameId){foundBySync=true;break}
        }
        if(foundBySync)continue;

        // Skip games the user intentionally deleted
        if(_isDeletedGame(rHash,rGameId))continue;

        var full=await fetchInternalGameContent(rGameId,rOwner||__relayUser);
        if(!full||typeof full.content!=='string')continue;
        var newId=uniqueLocalId(rGameId,col);
        col.games[newId]={
          name:full.name||rg.name||'Untitled',
          content:String(full.content||''),
          scope:'internal',
          _scope:'internal',
          version:full.version||rg.version||'',
          created:new Date().toISOString(),
          updated:new Date().toISOString(),
          _sync_owner:rOwner||String(__relayUser||'').trim().toLowerCase(),
          _sync_game_id:rGameId,
          _sync_hash:normHash(full.checksum||full.content_hash||rHash||''),
          checksum:normHash(full.checksum||full.content_hash||rHash||'')
        };
        localRows.push({
          id:newId,
          name:normName(col.games[newId].name||newId),
          size:String(full.content||'').length,
          hash:normHash(await shortHash(String(full.content||''))),
          game:col.games[newId]
        });
        changed=true;
      }

      for(var lid in col.games){
        var lg2=col.games[lid]||{};
        if(String(lg2._sync_game_id||'').trim())continue;
        if(!String(lg2.content||'').trim())continue;
        try{
          var pushed=await syncLocalToRelay(lid);
          changed=true;
          didLocalPush=true;
          if(pushed&&pushed.data){
            var po=String(pushed.data.owner||__relayUser||'').trim().toLowerCase();
            var pg=String(pushed.data.game_id||pushed.gameId||'').trim().toLowerCase();
            if(pg)mirrorKeys[po+'/'+pg]=true;
          }
        }catch(_){ }
      }

      if(didLocalPush){
        col=readGamesCollection();
      }

      // Mirror rule: be conservative.
      // Never prune internal/local games from a transient relay snapshot.
      // Only prune external mirror copies that are truly absent server-side.
      if(getToken()&&__relayUser){
        var removed=false;
        for(var rid in (col.games||{})){
          var rg2=col.games[rid]||{};
          var gid2=String(rg2._sync_game_id||'').trim().toLowerCase();
          var own2=String(rg2._sync_owner||__relayUser||'').trim().toLowerCase();
          var scope2=String((rg2._scope||rg2.scope||'internal')).trim().toLowerCase();
          // Keep true orphan entries locally until auto-sync succeeds.
          if(!gid2){continue}
          if(scope2!=='external'){continue}
          if(!mirrorKeys[own2+'/'+gid2]){delete col.games[rid];removed=true;continue}
        }
        if(removed)changed=true;
      }
      if(changed)writeGamesCollection(col);
    }finally{
      __reconcileInFlight=false;
    }
  }

  async function deleteRelayGame(gameId,name){
    if(!confirm('Delete "'+name+'" from server? This cannot be undone.'))return;
    var t=getToken();
    if(!t){alert('Login required.');return}
    _addDeletedGame('',gameId,name);
    try{
      var user='';try{var me=await fetch(RELAY+'/auth/me',{headers:authHeaders()});if(me.ok){var d=await me.json();user=d.username||''}}catch(_){}
      var delUrl=RELAY+'/internal/game/'+encodeURIComponent(gameId)+(user?('?owner='+encodeURIComponent(user)):'' );
      var r=await fetch(delUrl,{method:'DELETE',headers:authHeaders()});
      if(r.ok){
        var col=readGamesCollection();
        var changed=false;
        for(var id in (col.games||{})){
          var lg=col.games[id]||{};
          if(relayGameIdForLocal(id,lg)===String(gameId||'').toLowerCase()){
            delete col.games[id];
            if(col.active===id)col.active='';
            changed=true;
          }
        }
        if(changed)writeGamesCollection(col);
        await renderLocal();
        await renderRelay();
      }
      else{var d2=null;try{d2=await r.json()}catch(_){} alert((d2&&d2.error)||'Delete failed')}
    }catch(e){alert('Delete request failed')}
  }

  function makeLocalCard(g){
    var div=document.createElement('div');
    div.className='game-card'+(g.active?' active-game':'');
    var meta='';
    if(g.publishBadge){
      var badgeAttr=g.publishBadge.actionable?' data-publish-local="1"':'';
      meta+=' <span class="badge '+g.publishBadge.cls+'"'+badgeAttr+'>'+esc(g.publishBadge.label)+'</span>';
    }
    if(g.version) meta+=' <span style="opacity:0.25">\u00b7</span> '+esc(g.version);
    if(g.size) meta+=' <span style="opacity:0.25">\u00b7</span> '+fmtSize(g.size);
    var subMeta='';
    if(g.syncNote){subMeta='<div class="submeta">'+esc(g.syncNote)+'</div>'}
    div.innerHTML='<div class="gname">'+esc(g.name||'Untitled')+'</div>'
      +'<div class="gmeta">'+meta+'</div>'
      +subMeta
      +'<div class="gactions"><button class="btn-ren" data-ren="1">rename</button> <button class="btn-del" data-del="1">delete</button></div>'
      +'<div class="play-icon">\u25b6</div>';
    div.querySelector('[data-ren]').addEventListener('click',function(e){e.stopPropagation();renameLocalGame(g.id,g.name)});
    div.querySelector('[data-del]').addEventListener('click',function(e){e.stopPropagation();deleteLocalGame(g.id,g.name)});
    var pubBtn=div.querySelector('[data-publish-local]');
    console.log('[games:makeLocalCard]',g.name,'pubBtn=',!!pubBtn,'badge=',JSON.stringify(g.publishBadge),'gameId=',g.gameId,'isPublished=',g.isPublished,'offline=',g.offline,'unsynced=',g.unsynced);
    if(pubBtn){
      pubBtn.addEventListener('click',function(e){
        e.stopPropagation();
        console.log('[games:pubClick]',g.name,'offline=',g.offline,'unsynced=',g.unsynced,'gameId=',g.gameId,'isPublished=',g.isPublished);
        if(g.offline){
          alert('Relay unavailable ('+(__relayHealth.status||0)+'). Try again once relay.slob.games is reachable.');
          return;
        }
        if(g.unsynced){
          console.log('[games:pubClick] syncing to relay...',g.id);
          syncLocalToRelay(g.id).then(async function(){
            await renderLocal();
            await renderRelay();
          }).catch(function(err){console.error('[games:pubClick] sync failed:',err);alert((err&&err.message)||'Sync failed')});
          return;
        }
        if(g.gameId){
          console.log('[games:pubClick] toggling published:',g.gameId,'->',!g.isPublished);
          setPublished(g.gameId,!g.isPublished);
          return;
        }
        console.log('[games:pubClick] publishLocalGame:',g.id);
        publishLocalGame(g.id);
      });
    }
    div.addEventListener('click',function(){activateAndGoCanvas(g.id,g.game||null)});
    return div;
  }

  function makeRelayCard(g){
    var div=document.createElement('div');
    div.className='game-card';
    var isPub=g.published!==undefined?!!g.published:true;
    var pubBadge=isPub?'<span class="badge pub" data-pub="1">published</span>':'<span class="badge draft" data-pub="1">draft</span>';
    var meta='<span class="badge relay">yours</span> '+pubBadge;
    if(g.size) meta+=' <span style="opacity:0.25">\u00b7</span> '+fmtSize(g.size);
    div.innerHTML='<div class="gname">'+esc(g.name||'Untitled')+'</div>'
      +'<div class="gmeta">'+meta+'</div>'
      +'<div class="gactions"><button class="btn-ren" data-ren="1">rename</button> <button class="btn-del" data-del="1">delete</button></div>'
      +'<div class="play-icon">\u25b6</div>';
    div.querySelector('[data-pub]').addEventListener('click',function(e){e.stopPropagation();setPublished(g.game_id,!isPub)});
    div.querySelector('[data-ren]').addEventListener('click',function(e){e.stopPropagation();renameRelayGame(g.game_id,g.name,g.content_hash)});
    div.querySelector('[data-del]').addEventListener('click',function(e){e.stopPropagation();deleteRelayGame(g.game_id,g.name)});
    div.addEventListener('click',function(){playOrFetch(g.content_hash,g.name)});
    return div;
  }

  function makeCommunityCard(g){
    var div=document.createElement('div');
    div.className='game-card';
    var chash=normHash(g.content_hash||'');
    var status=getGameVfsStatus(chash);
    var meta='<span class="status-dot '+status+'" data-hash="'+esc(chash)+'"></span> <span class="badge relay">community</span>';
    if(g.size) meta+=' <span style="opacity:0.25">\u00b7</span> '+fmtSize(g.size);
    div.innerHTML='<div class="gname">'+esc(g.name||'Untitled')+'</div>'
      +'<div class="gmeta">'+meta+'</div>'
      +'<div class="play-icon">\u25b6</div>';
    div.addEventListener('click',function(){playOrFetch(g.content_hash,g.name)});
    return div;
  }

  function playFromVfs(contentHash,name){
    var hash=normHash(contentHash);
    if(!hash)return false;
    var col=readGamesCollection();
    for(var id in (col.games||{})){
      var g=col.games[id]||{};
      var h=normHash(g._sync_hash||g.checksum||'');
      if(h&&h===hash&&g.content){
        activateAndGoCanvas(id,g);
        return true;
      }
    }
    return false;
  }

  function playOrFetch(contentHash,name){
    if(playFromVfs(contentHash,name))return;
    fetchAndPlay(contentHash,name);
  }

  async function renderLocal(){
    var grid=document.getElementById('localGrid');
    var hasToken=!!getToken();
    console.log('[games:renderLocal] hasToken=',hasToken,'relayUser=',__relayUser,'relayHealth=',JSON.stringify(__relayHealth),'relayMineByGameId keys=',Object.keys(__relayMineByGameId),'relayMineByOwnerGameId keys=',Object.keys(__relayMineByOwnerGameId));
    var col=readGamesCollection();
    var byIdentity={};
    for(var id in (col.games||{})){
      var g=col.games[id];
      var row={
        id:id,
        name:g.name||'Untitled',
        version:g.version||'',
        size:(g.content||'').length,
        scope:g.scope||'internal',
        active:id===col.active,
        updated:g.updated||'',
        game:g
      };
      var ident=String((g&&g._sync_game_id)||id||'untitled').trim().toLowerCase();
      var prev=byIdentity[ident];
      if(!prev){
        byIdentity[ident]=row;
        continue;
      }
      if(row.active&&!prev.active){
        byIdentity[ident]=row;
        continue;
      }
      if(!row.active&&prev.active){
        continue;
      }
      if(String(row.updated||'')>String(prev.updated||'')){
        byIdentity[ident]=row;
      }
    }
    var list=[];
    for(var identKey in byIdentity)list.push(byIdentity[identKey]);
    list.sort(function(a,b){
      var an=String(a.name||'untitled').toLowerCase();
      var bn=String(b.name||'untitled').toLowerCase();
      if(an!==bn)return an.localeCompare(bn);
      return String(a.id||'').localeCompare(String(b.id||''));
    });
    grid.innerHTML='';
    if(!list.length){grid.innerHTML='<div class="empty">No local games yet. Use the canvas to create one.</div>';return}

    var enriched=[];
    for(var i=0;i<list.length;i++){
      var g=list[i];
      var raw=g.game||{};
      var localScope=String((raw._scope||raw.scope||'internal')).trim().toLowerCase();
      var isExternalLocal=(localScope==='external');
      var hasSyncIdentity=!!String((raw._sync_game_id||'')).trim();
      var gameId=relayGameIdForLocal(g.id,raw);
      var syncOwner=String((raw._sync_owner||'')).trim().toLowerCase();
      var ownerForLookup=syncOwner||String(__relayUser||'').trim().toLowerCase();
      var relayKey=ownerForLookup?(ownerForLookup+'/'+gameId):'';
      var relay=relayKey?(__relayMineByOwnerGameId[relayKey]||null):null;
      var localHash='';
      try{localHash=await shortHash(raw.content||'')}catch(_){localHash=''}
      var relayHash=String((relay&&(relay.checksum||relay.content_hash))||raw._sync_hash||raw.checksum||'').trim().toLowerCase();
      var isPublished=!!(relay&&relay.published);
      g.gameId=gameId;
      g.isPublished=isPublished;
      g.offline=!__relayHealth.ok;
      g.unsynced=!isExternalLocal&&hasToken&&!hasSyncIdentity&&!relay&&__relayHealth.ok;

      if(g.offline){
        g.publishBadge={cls:'offline',label:'offline',actionable:false};
      }else if(isExternalLocal){
        g.publishBadge={cls:'ext',label:'community copy',actionable:false};
      }else if(!hasToken){
        g.publishBadge=hasSyncIdentity
          ?{cls:'draft',label:'draft',actionable:false}
          :{cls:'local',label:'local',actionable:false};
      }else if(hasSyncIdentity&&!relay){
        g.publishBadge={cls:'draft',label:'draft',actionable:true};
      }else if(!relay){
        g.publishBadge={cls:'publish-dim',label:'unsynced',actionable:true};
      }else if(isPublished){
        g.publishBadge={cls:'pub',label:'published',actionable:true};
      }else{
        g.publishBadge={cls:'draft',label:'draft',actionable:true};
      }

      console.log('[games:renderLocal] game=',g.name,'id=',g.id,'scope=',localScope,'isExternalLocal=',isExternalLocal,'hasSyncIdentity=',hasSyncIdentity,'gameId=',gameId,'syncOwner=',syncOwner,'ownerForLookup=',ownerForLookup,'relayKey=',relayKey,'relay=',relay?{name:relay.name,published:relay.published,checksum:relay.checksum}:null,'badge=',JSON.stringify(g.publishBadge));
      g.syncNote='';
      enriched.push(g);
    }

    enriched.sort(function(a,b){
      var an=String(a.name||'untitled').trim().toLowerCase();
      var bn=String(b.name||'untitled').trim().toLowerCase();
      if(an!==bn)return an.localeCompare(bn);
      return String(a.id||'').localeCompare(String(b.id||''));
    });

    enriched.forEach(function(g){grid.appendChild(makeLocalCard(g))});
  }

  async function renderRelay(){
    var grid=document.getElementById('relayGrid');
    var t=getToken();
    console.log('[games:renderRelay] hasToken=',!!t,'RELAY=',RELAY);
    __relayMineByGameId={};
    __relayMineByOwnerGameId={};
    __relayUser='';
    _setRelayHealth(true,200,'');

    var publicResp=await fetchJson('/games');
    var publicGames=[];
    if(!publicResp.ok){
      // Relay down — try GitHub catalog fallback
      var hint='Could not load community games.';
      if(publicResp.status===530||/1016/.test(String(publicResp.text||''))){
        hint='Relay unavailable (Cloudflare 1016 / 530). Check relay domain routing.';
      }else if(publicResp.status){
        hint='Could not load community games ('+publicResp.status+').';
      }
      if(__githubCatalogUrl){
        try{
          var ghResp=await fetch(__githubCatalogUrl,{cache:'no-store'});
          if(ghResp.ok){
            var ghData=await ghResp.json().catch(function(){return null});
            if(ghData&&Array.isArray(ghData.games)&&ghData.games.length){
              publicGames=ghData.games;
              console.log('[games:renderRelay] using GitHub catalog fallback, games=',publicGames.length);
              hint='';
            }
          }
        }catch(_){}
      }
      if(!publicGames.length){
        _setRelayHealth(false,publicResp.status,hint);
        grid.innerHTML='<div class="empty">'+esc(hint)+' <a href="#" style="color:#00e0ff;font-size:0.75em" onclick="this.closest(\'div\').textContent=\'Retrying…\';renderRelay().catch(function(){renderLocal()});return false">Retry</a></div>';
        await renderLocal();
        return;
      }
      _setRelayHealth(false,publicResp.status,'Relay offline — showing GitHub catalog');
    }else{
      publicGames=Array.isArray(publicResp.data)?publicResp.data:[];
    }

    // If logged in, show user's own games with publish/delete controls
    if(t){
      try{
        var meResp=await fetchJson('/auth/me',authHeaders());
        if(meResp.ok&&meResp.data){
          __relayUser=String(meResp.data.username||'').trim().toLowerCase();
        }

        var myResp=await fetchJson('/internal/games',authHeaders());
        var myGames=[];
        if(myResp.ok){
          myGames=Array.isArray(myResp.data)?myResp.data:[];
        }else if(myResp.status===401){
          // stale token in this browser context; continue as logged-out view
          __relayUser='';
        }else{
          // Relay is online (public games loaded) but private-games fetch failed.
          // Don't mark relay offline — show a warning but keep health ok so local
          // games don't get the misleading "offline" badge.
          console.warn('[games:renderRelay] /internal/games failed:',myResp.status,myResp.text);
          grid.innerHTML='<div class="empty">Could not load your games ('+myResp.status+'). <a href="#" style="color:#00e0ff;font-size:0.75em" onclick="renderRelay().catch(function(){renderLocal()});return false">Retry</a></div>';
          await renderLocal();
          return;
        }

        if(myGames){
          console.log('[games:renderRelay] myGames count=',myGames.length,'relayUser=',__relayUser);
          myGames.forEach(function(mg){console.log('[games:renderRelay] myGame:',mg.name,'game_id=',mg.game_id,'owner=',mg.owner,'published=',mg.published,'checksum=',mg.checksum)});
          // Do not auto-merge by name during normal render flow.
          // It is destructive (can rewrite IDs/delete variants) and can hide
          // a just-renamed game before relay state settles.
          await reconcileLocalAndRelay(myGames);
          var rows=[];
          myGames.forEach(function(g){
            var gid=slugify(g.game_id||g.name||'untitled');
            var owner=String((g.owner||__relayUser||'')).trim().toLowerCase();
            __relayMineByGameId[gid]=g;
            if(owner&&gid)__relayMineByOwnerGameId[owner+'/'+gid]=g;
            console.log('[games:renderRelay] indexed:', owner+'/'+gid, 'published=',g.published);
            // Render user's own games with draft/publish toggle
            rows.push({kind:'relay',name:String(g.name||'Untitled'),game:g});
          });
          // Show the full published community catalog regardless of auth state.
          if(publicGames.length){
            publicGames.forEach(function(g){rows.push({kind:'community',name:String(g.name||'Untitled'),game:g})});
          }
          grid.innerHTML='';
          rows.sort(function(a,b){return a.name.toLowerCase().localeCompare(b.name.toLowerCase())});
          rows.forEach(function(r){
            var card = r.kind==='relay' ? makeRelayCard(r.game) : makeCommunityCard(r.game);
            grid.appendChild(card);
          });
          if(!grid.children.length)grid.innerHTML='<div class="empty">No community games available.</div>';
          _injectPublishAllBtn();
          await renderLocal();
          return;
        }
      }catch(e){}
    }
    // Fallback: not logged in — show all community games
    grid.innerHTML='';
    if(!publicGames.length){grid.innerHTML='<div class="empty">No community games available.</div>';await renderLocal();return}
    publicGames.sort(function(a,b){return String(a.name||'').localeCompare(String(b.name||''))});
    publicGames.forEach(function(g){grid.appendChild(makeCommunityCard(g))});
    await renderLocal();
  }

  async function publishAllToGitHub(btn){
    var gameIds=Object.keys(__relayMineByGameId);
    if(!gameIds.length){alert('No games on relay to publish.');return}
    if(btn)btn.disabled=true;
    var ok=0,fail=0,errors=[];
    for(var i=0;i<gameIds.length;i++){
      try{
        var r=await fetch(RELAY+'/internal/game/'+encodeURIComponent(gameIds[i])+'/github-publish',{
          method:'PATCH',headers:authHeaders(),body:JSON.stringify({})
        });
        var d=null;try{d=await r.json()}catch(_){}
        if(r.ok)ok++;
        else{fail++;errors.push(gameIds[i]+': '+(d&&d.error||r.status))}
      }catch(e){fail++;errors.push(gameIds[i]+': '+String(e&&e.message||e))}
    }
    if(btn)btn.disabled=false;
    var msg='Published '+ok+' game'+(ok!==1?'s':'')+ ' to GitHub.';
    if(fail)msg+='\n'+fail+' failed:\n'+errors.join('\n');
    alert(msg);
  }

  function _injectPublishAllBtn(){
    var sec=document.getElementById('yourGamesSection');
    if(!sec)return;
    if(document.getElementById('publishAllBtn'))return;
    var h2=sec.querySelector('h2');
    if(!h2)return;
    var btn=document.createElement('button');
    btn.id='publishAllBtn';
    btn.className='publish-all-btn';
    btn.textContent='↑ GitHub';
    btn.title='Publish all your games to GitHub catalog';
    btn.onclick=function(){publishAllToGitHub(btn)};
    h2.parentNode.insertBefore(btn,h2.nextSibling);
  }

  async function fetchAndPlay(hash,name){
    var hashKey=normHash(hash);
    if(hashKey){__fetchingHashes[hashKey]=true;updateStatusDots()}
    try{
      var content=null;
      var gameName=name||'Game';
      // Try P2P download first (relay WS + peer mesh, 3s timeout)
      if(window.__requestGameP2P){
        try{
          var p2p=await window.__requestGameP2P(hash,3000);
          if(p2p&&p2p.content){
            content=p2p.content;
            gameName=p2p.name||gameName;
            console.log('[games] loaded via P2P:',gameName);
          }
        }catch(_){}
      }
      // Fall back to REST endpoint (with relay failover)
      if(!content){
        var r=await fetchJson('/game/'+hash);
        if(r.ok&&r.data&&r.data.content){
          content=r.data.content;
          gameName=r.data.name||gameName;
        }
      }
      // Retry once if first attempt failed (transient network/CORS errors)
      if(!content){
        await new Promise(function(ok){setTimeout(ok,500)});
        var r2=await fetchJson('/game/'+hash);
        if(r2.ok&&r2.data&&r2.data.content){
          content=r2.data.content;
          gameName=r2.data.name||gameName;
        }
      }
      // Last resort: look up in GitHub catalog by checksum, then fetch raw content
      if(!content&&__githubCatalogUrl){
        try{
          var ghIdx=await fetch(__githubCatalogUrl,{cache:'no-store'});
          if(ghIdx.ok){
            var ghIdxData=await ghIdx.json().catch(function(){return null});
            var normH=String(hash||'').trim().toLowerCase();
            var entry=(ghIdxData&&Array.isArray(ghIdxData.games))
              ?ghIdxData.games.find(function(g){return String(g.checksum||g.content_hash||'').trim().toLowerCase()===normH})
              :null;
            if(entry&&entry.owner&&entry.game_id){
              var rawBase=__githubCatalogUrl.replace(/\/games\/index\.json$/,'');
              var rawUrl=rawBase+'/games/'+encodeURIComponent(entry.owner)+'/'+encodeURIComponent(entry.game_id)+'.json';
              var rawResp=await fetch(rawUrl,{cache:'no-store'});
              if(rawResp.ok){
                var rawGame=await rawResp.json().catch(function(){return null});
                if(rawGame&&rawGame.content){content=rawGame.content;gameName=rawGame.name||gameName;console.log('[games] loaded via GitHub fallback:',gameName);}
              }
            }
          }
        }catch(_){}
      }
      if(!content)return;
      var safeHash=String(hash||'').trim().toLowerCase();
      var id='s-'+safeHash.substring(0,16);
      var gameId=safeHash.substring(0,16)||slugify(gameName||'game');
      // Build game object and use activateAndGoCanvas which properly
      // sets the launch payload so canvas renders the correct game
      var gameObj={
        name:gameName,
        content:content,
        _scope:'external',
        _sync_owner:'community',
        _sync_game_id:gameId,
        _sync_hash:safeHash,
        version:''
      };
      await activateAndGoCanvas(id,gameObj);
    }catch(e){console.error('Failed to load game:',e)}
    finally{if(hashKey)delete __fetchingHashes[hashKey];updateStatusDots()}
  }

  ensureRelayBase().finally(function(){
    renderRelay().catch(function(){renderLocal()});
  });
})();
"##;
