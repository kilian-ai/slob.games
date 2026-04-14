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
                    div.games-section {
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
.submeta{font-size:0.62rem;color:#6f7f96;opacity:.9}
.btn-del{background:none;border:1px solid rgba(255,60,60,0.2);color:#ff4444;font-size:0.6rem;padding:1px 6px;border-radius:3px;cursor:pointer;text-transform:uppercase;letter-spacing:0.04em;font-weight:600;transition:all 0.2s}
.btn-del:hover{background:rgba(255,60,60,0.12);border-color:rgba(255,60,60,0.4)}
.play-icon{position:absolute;top:50%;right:0.75rem;transform:translateY(-50%);width:28px;height:28px;border-radius:50%;background:rgba(0,224,255,0.08);border:1px solid rgba(0,224,255,0.15);color:#00e0ff;display:flex;align-items:center;justify-content:center;font-size:0.75rem;opacity:0;transition:opacity 0.2s}
.game-card:hover .play-icon{opacity:1}
.game-card.active-game{border-color:rgba(0,255,136,0.25)}
.game-card.active-game .gname{color:#00ff88}
.empty{color:#5a6570;font-size:0.82rem;font-style:italic;padding:0.5rem 0}
.loading{color:#5a6570;font-size:0.82rem;padding:0.5rem 0}
@media(max-width:640px){.games-grid{grid-template-columns:1fr}.games-page{padding:1.5rem 1rem}}
"##;

const JS: &str = r##"
(function(){
  function esc(s){var d=document.createElement('div');d.textContent=s;return d.innerHTML}
  function fmtSize(b){return b<1024?b+'B':(b/1024).toFixed(1)+'KB'}
  var RELAY='https://relay.slob.games/sync';
  var __relayMineByGameId={};
  var __relayUser='';
  var __reconcileInFlight=false;

  function getToken(){return(localStorage.getItem('traits.secret.SLOB_USER_TOKEN')||'').trim()}
  function authHeaders(){var h={'Content-Type':'application/json'};var t=getToken();if(t)h['Authorization']='Bearer '+t;return h}
  function slugify(s){
    return String(s||'').trim().toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-+|-+$/g,'')||'untitled';
  }
  function relayGameIdForLocal(id,g){
    return String((g&&(g._sync_game_id||g.game_id))||slugify((g&&g.name)||id||'untitled')).trim().toLowerCase();
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

  function readGamesCollection(){
    try{
      var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
      var raw=pvfs['canvas/games.json'];
      if(raw)return JSON.parse(raw);
    }catch(e){}
    return {active:'',games:{}};
  }

  function writeGamesCollection(col){
    var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
    pvfs['canvas/games.json']=JSON.stringify(col);
    localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
  }

  function setActiveGame(id){
    var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
    var col=pvfs['canvas/games.json']?JSON.parse(pvfs['canvas/games.json']):{active:'',games:{}};
    col.active=id;
    if(col.games[id]&&col.games[id].content){
      pvfs['canvas/app.html']=col.games[id].content;
    }
    pvfs['canvas/games.json']=JSON.stringify(col);
    localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
  }

  async function activateAndGoCanvas(id){
    // Persist selection first so canvas route always opens the clicked game,
    // even if SDK activation is delayed or unavailable.
    setActiveGame(id);
    var sdk=window._traitsSDK;
    if(sdk){
      try{await sdk.call('sys.canvas',['activate',id])}catch(_){}
    }
    goCanvas();
  }

  function goCanvas(){
    window.dispatchEvent(new CustomEvent('traits-spa-action',{detail:{spa_action:'navigate',route:'/'}}));
  }

  async function deleteLocalGame(id,name){
    if(!confirm('Delete "'+name+'"? This cannot be undone.'))return;
    var col=readGamesCollection();
    var game=col.games[id]||{};
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
    if(!t){alert('Login required to publish/unpublish.');return}
    try{
      var r=await fetch(RELAY+'/internal/game/'+encodeURIComponent(gameId)+'/publish',{
        method:'PATCH',headers:authHeaders(),body:JSON.stringify({published:!!published})
      });
      if(r.ok){await renderRelay();await renderLocal();}
      else{var d=null;try{d=await r.json()}catch(_){} alert((d&&d.error)||'Publish update failed')}
    }catch(e){alert('Publish request failed')}
  }

  async function syncLocalToRelay(localId){
    var t=getToken();
    if(!t){alert('Login required to publish.');return null}
    var col=readGamesCollection();
    var g=col.games[localId];
    if(!g||!g.content){alert('Nothing to publish for this game.');return null}
    // Filename (game_id) is the stable identity; mirror updates overwrite that key.
    var gameId=relayGameIdForLocal(localId,g);
    var payload={name:g.name||'Untitled',content:String(g.content||''),version:g.version||'',scope:'internal'};
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
    return {gameId:gameId,data:data};
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

      // Mirror rule: when logged in, keep only relay-linked entries for this user.
      if(getToken()&&__relayUser){
        var removed=false;
        for(var rid in (col.games||{})){
          var rg2=col.games[rid]||{};
          var gid2=String(rg2._sync_game_id||'').trim().toLowerCase();
          var own2=String(rg2._sync_owner||__relayUser||'').trim().toLowerCase();
          // Keep true orphan entries locally until auto-sync succeeds.
          if(!gid2){continue}
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
      meta+=' <span class="badge '+g.publishBadge.cls+'" data-publish-local="1">'+esc(g.publishBadge.label)+'</span>';
    }
    if(g.version) meta+=' <span style="opacity:0.25">\u00b7</span> '+esc(g.version);
    if(g.size) meta+=' <span style="opacity:0.25">\u00b7</span> '+fmtSize(g.size);
    var subMeta='';
    if(g.syncNote){subMeta='<div class="submeta">'+esc(g.syncNote)+'</div>'}
    div.innerHTML='<div class="gname">'+esc(g.name||'Untitled')+'</div>'
      +'<div class="gmeta">'+meta+'</div>'
      +subMeta
      +'<div class="gactions"><button class="btn-del" data-del="1">delete</button></div>'
      +'<div class="play-icon">\u25b6</div>';
    div.querySelector('[data-del]').addEventListener('click',function(e){e.stopPropagation();deleteLocalGame(g.id,g.name)});
    var pubBtn=div.querySelector('[data-publish-local]');
    if(pubBtn){
      pubBtn.addEventListener('click',function(e){
        e.stopPropagation();
        if(g.unsynced){
          syncLocalToRelay(g.id).then(function(){renderRelay()}).catch(function(err){alert((err&&err.message)||'Sync failed')});
          return;
        }
        if(g.gameId){
          setPublished(g.gameId,!g.isPublished);
          return;
        }
        publishLocalGame(g.id);
      });
    }
    div.addEventListener('click',function(){activateAndGoCanvas(g.id)});
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
      +'<div class="gactions"><button class="btn-del" data-del="1">delete</button></div>'
      +'<div class="play-icon">\u25b6</div>';
    div.querySelector('[data-pub]').addEventListener('click',function(e){e.stopPropagation();setPublished(g.game_id,!isPub)});
    div.querySelector('[data-del]').addEventListener('click',function(e){e.stopPropagation();deleteRelayGame(g.game_id,g.name)});
    div.addEventListener('click',function(){fetchAndPlay(g.content_hash,g.name)});
    return div;
  }

  function makeCommunityCard(g){
    var div=document.createElement('div');
    div.className='game-card';
    var meta='<span class="badge relay">community</span>';
    if(g.size) meta+=' <span style="opacity:0.25">\u00b7</span> '+fmtSize(g.size);
    div.innerHTML='<div class="gname">'+esc(g.name||'Untitled')+'</div>'
      +'<div class="gmeta">'+meta+'</div>'
      +'<div class="play-icon">\u25b6</div>';
    div.addEventListener('click',function(){fetchAndPlay(g.content_hash,g.name)});
    return div;
  }

  async function renderLocal(){
    var grid=document.getElementById('localGrid');
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
      var ident=slugify(g.name||id||'untitled');
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
      var gameId=relayGameIdForLocal(g.id,raw);
      var relay=__relayMineByGameId[gameId]||null;
      var localHash='';
      try{localHash=await shortHash(raw.content||'')}catch(_){localHash=''}
      var relayHash=String((relay&&(relay.checksum||relay.content_hash))||raw._sync_hash||raw.checksum||'').trim().toLowerCase();
      var inSync=!!(relay&&localHash&&relayHash&&localHash===relayHash);
      var isPublished=!!(relay&&relay.published);
      g.gameId=gameId;
      g.isPublished=isPublished;
      g.unsynced=!relay;

      if(!relay){
        g.publishBadge={cls:'publish-dim',label:'unsynced'};
      }else if(isPublished){
        g.publishBadge={cls:'pub',label:'published'};
      }else{
        g.publishBadge={cls:'draft',label:'draft'};
      }

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
    __relayMineByGameId={};
    __relayUser='';
    // If logged in, show user's own games with publish/delete controls
    if(t){
      try{
        try{
          var me=await fetch(RELAY+'/auth/me',{headers:authHeaders()});
          if(me.ok){var meData=await me.json();__relayUser=String(meData.username||'').trim().toLowerCase()}
        }catch(_){}
        var res=await fetch(RELAY+'/internal/games',{headers:authHeaders()});
        if(res.ok){
          var myGames=await res.json();
          await reconcileLocalAndRelay(myGames);
          var myIds={};
          var myKeys={};
          var rows=[];
          myGames.forEach(function(g){
            var gid=slugify(g.game_id||g.name||'untitled');
            __relayMineByGameId[gid]=g;
            if(!!g.published){
              myIds[String(g.content_hash||'').trim().toLowerCase()]=true;
              myKeys[String((g.owner||__relayUser||'')).trim().toLowerCase()+'/'+gid]=true;
            }
          });
          // Also show other community games below
          var res2=await fetch(RELAY+'/games');
          if(res2.ok){
            var community=await res2.json();
            var others=community.filter(function(g){
              var owner=String(g.owner||'').trim().toLowerCase();
              var gid=slugify(g.game_id||g.name||'untitled');
              var key=owner+'/'+gid;
              var hash=String(g.content_hash||'').trim().toLowerCase();
              return !myIds[hash] && !myKeys[key];
            });
            if(others.length){
              others.forEach(function(g){rows.push({kind:'community',name:String(g.name||'Untitled'),game:g})});
            }
          }
          grid.innerHTML='';
          rows.sort(function(a,b){return a.name.toLowerCase().localeCompare(b.name.toLowerCase())});
          rows.forEach(function(r){
            grid.appendChild(makeCommunityCard(r.game));
          });
          if(!grid.children.length)grid.innerHTML='<div class="empty">No community games available.</div>';
          await renderLocal();
          return;
        }
      }catch(e){}
    }
    // Fallback: not logged in — show all community games
    try{
      var res=await fetch(RELAY+'/games');
      var games=await res.json();
      grid.innerHTML='';
      if(!games.length){grid.innerHTML='<div class="empty">No community games available.</div>';return}
      games.sort(function(a,b){return a.name.localeCompare(b.name)});
      games.forEach(function(g){grid.appendChild(makeCommunityCard(g))});
    }catch(e){
      grid.innerHTML='<div class="empty">Could not load community games.</div>';
    }
    await renderLocal();
  }

  async function fetchAndPlay(hash,name){
    try{
      var res=await fetch(RELAY+'/game/'+hash);
      var data=await res.json();
      if(!data.content)return;
      var col=readGamesCollection();
      var id='s-'+hash.substring(0,16);
      col.games[id]={name:name,content:data.content,scope:'external',version:'',created:new Date().toISOString(),updated:new Date().toISOString()};
      col.active=id;
      writeGamesCollection(col);
      await activateAndGoCanvas(id);
    }catch(e){console.error('Failed to load game:',e)}
  }

  renderRelay().catch(function(){renderLocal()});
})();
"##;
