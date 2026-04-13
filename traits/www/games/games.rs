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

  function getToken(){return(localStorage.getItem('traits.secret.SLOB_USER_TOKEN')||'').trim()}
  function authHeaders(){var h={'Content-Type':'application/json'};var t=getToken();if(t)h['Authorization']='Bearer '+t;return h}

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

  function goCanvas(){
    window.dispatchEvent(new CustomEvent('traits-spa-action',{detail:{spa_action:'navigate',route:'/'}}));
  }

  function deleteLocalGame(id,name){
    if(!confirm('Delete "'+name+'"? This cannot be undone.'))return;
    var col=readGamesCollection();
    delete col.games[id];
    if(col.active===id)col.active='';
    writeGamesCollection(col);
    renderLocal();
  }

  async function togglePublish(gameId){
    var t=getToken();
    if(!t){alert('Login required to publish/unpublish.');return}
    try{
      var r=await fetch(RELAY+'/internal/game/'+encodeURIComponent(gameId)+'/publish',{method:'PATCH',headers:authHeaders()});
      if(r.ok){renderRelay()}
      else{var d=null;try{d=await r.json()}catch(_){} alert((d&&d.error)||'Toggle failed')}
    }catch(e){alert('Toggle request failed')}
  }

  async function deleteRelayGame(gameId,name){
    if(!confirm('Delete "'+name+'" from server? This cannot be undone.'))return;
    var t=getToken();
    if(!t){alert('Login required.');return}
    try{
      var user='';try{var me=await fetch(RELAY+'/auth/me',{headers:authHeaders()});if(me.ok){var d=await me.json();user=d.username||''}}catch(_){}
      var r=await fetch(RELAY+'/internal/game/'+encodeURIComponent(gameId)+'?owner='+encodeURIComponent(user),{method:'DELETE',headers:authHeaders()});
      if(r.ok){renderRelay()}
      else{var d2=null;try{d2=await r.json()}catch(_){} alert((d2&&d2.error)||'Delete failed')}
    }catch(e){alert('Delete request failed')}
  }

  function makeLocalCard(g){
    var div=document.createElement('div');
    div.className='game-card'+(g.active?' active-game':'');
    var badgeCls=g.scope==='external'?'ext':'local';
    var badgeLabel=g.scope==='external'?'synced':'local';
    var meta='<span class="badge '+badgeCls+'">'+badgeLabel+'</span>';
    if(g.version) meta+=' <span style="opacity:0.25">\u00b7</span> '+esc(g.version);
    if(g.size) meta+=' <span style="opacity:0.25">\u00b7</span> '+fmtSize(g.size);
    div.innerHTML='<div class="gname">'+esc(g.name||'Untitled')+'</div>'
      +'<div class="gmeta">'+meta+'</div>'
      +'<div class="gactions"><button class="btn-del" data-del="1">delete</button></div>'
      +'<div class="play-icon">\u25b6</div>';
    div.querySelector('[data-del]').addEventListener('click',function(e){e.stopPropagation();deleteLocalGame(g.id,g.name)});
    div.addEventListener('click',function(){setActiveGame(g.id);goCanvas()});
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
    div.querySelector('[data-pub]').addEventListener('click',function(e){e.stopPropagation();togglePublish(g.game_id)});
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

  function renderLocal(){
    var grid=document.getElementById('localGrid');
    var col=readGamesCollection();
    var all=[];
    for(var id in (col.games||{})){
      var g=col.games[id];
      all.push({id:id,name:g.name||'Untitled',version:g.version||'',size:(g.content||'').length,scope:g.scope||'internal',active:id===col.active,updated:g.updated||''});
    }
    var byName={};
    all.forEach(function(g){
      var key=(g.name||'').toLowerCase().trim();
      var prev=byName[key];
      if(!prev){byName[key]=g;return}
      if(g.active&&!prev.active){byName[key]=g;return}
      if(!g.active&&prev.active)return;
      if((g.updated||'')>(prev.updated||''))byName[key]=g;
    });
    var list=[];for(var k in byName)list.push(byName[k]);
    list.sort(function(a,b){return (b.active?1:0)-(a.active?1:0)||(b.updated||'').localeCompare(a.updated||'')});
    grid.innerHTML='';
    if(!list.length){grid.innerHTML='<div class="empty">No local games yet. Use the canvas to create one.</div>';return}
    list.forEach(function(g){grid.appendChild(makeLocalCard(g))});
  }

  async function renderRelay(){
    var grid=document.getElementById('relayGrid');
    var t=getToken();
    // If logged in, show user's own games with publish/delete controls
    if(t){
      try{
        var res=await fetch(RELAY+'/internal/games',{headers:authHeaders()});
        if(res.ok){
          var myGames=await res.json();
          var myIds={};
          if(myGames.length){
            grid.innerHTML='';
            myGames.sort(function(a,b){return(a.name||'').localeCompare(b.name||'')});
            myGames.forEach(function(g){myIds[g.content_hash]=true;grid.appendChild(makeRelayCard(g))});
          }
          // Also show other community games below
          var res2=await fetch(RELAY+'/games');
          if(res2.ok){
            var community=await res2.json();
            var others=community.filter(function(g){return !myIds[g.content_hash]});
            if(others.length){
              others.sort(function(a,b){return(a.name||'').localeCompare(b.name||'')});
              others.forEach(function(g){grid.appendChild(makeCommunityCard(g))});
            }
          }
          if(!grid.children.length)grid.innerHTML='<div class="empty">No community games available.</div>';
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
      goCanvas();
    }catch(e){console.error('Failed to load game:',e)}
  }

  renderLocal();
  renderRelay();
})();
"##;
