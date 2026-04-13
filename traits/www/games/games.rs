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
.badge{font-size:0.6rem;padding:1px 5px;border-radius:3px;text-transform:uppercase;letter-spacing:0.04em;font-weight:600}
.badge.local{background:rgba(0,255,136,0.08);color:#00ff88}
.badge.ext{background:rgba(0,224,255,0.08);color:#00e0ff}
.badge.relay{background:rgba(255,45,120,0.08);color:#ff2d78}
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

  function readGamesCollection(){
    try{
      var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
      var raw=pvfs['canvas/games.json'];
      if(raw)return JSON.parse(raw);
    }catch(e){}
    return {active:'',games:{}};
  }

  function setActiveGame(id){
    var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
    var col=pvfs['canvas/games.json']?JSON.parse(pvfs['canvas/games.json']):{active:'',games:{}};
    col.active=id;
    pvfs['canvas/games.json']=JSON.stringify(col);
    localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
  }

  function goCanvas(){
    window.dispatchEvent(new CustomEvent('traits-spa-action',{detail:{spa_action:'navigate',route:'/'}}));
  }

  function makeCard(name,scope,version,size,isActive){
    var div=document.createElement('div');
    div.className='game-card'+(isActive?' active-game':'');
    var badgeCls=scope==='external'?'ext':(scope==='relay'?'relay':'local');
    var badgeLabel=scope==='external'?'synced':(scope==='relay'?'community':'local');
    var meta=[
      '<span class="badge '+badgeCls+'">'+badgeLabel+'</span>',
      version?esc(version):'',
      size?fmtSize(size):''
    ].filter(Boolean).join(' <span style="opacity:0.25">\u00b7</span> ');
    div.innerHTML='<div class="gname">'+esc(name||'Untitled')+'</div>'
      +'<div class="gmeta">'+meta+'</div>'
      +'<div class="play-icon">\u25b6</div>';
    return div;
  }

  // Local games (deduplicated by name — keep latest updated per name)
  (function loadLocal(){
    var grid=document.getElementById('localGrid');
    var col=readGamesCollection();
    var all=[];
    for(var id in (col.games||{})){
      var g=col.games[id];
      all.push({id:id,name:g.name||'Untitled',version:g.version||'',size:(g.content||'').length,scope:g.scope||'internal',active:id===col.active,updated:g.updated||''});
    }
    // Deduplicate: group by lowercase name, keep the best entry per name
    var byName={};
    all.forEach(function(g){
      var key=(g.name||'').toLowerCase().trim();
      var prev=byName[key];
      if(!prev){byName[key]=g;return}
      // prefer active, then most recently updated
      if(g.active&&!prev.active){byName[key]=g;return}
      if(!g.active&&prev.active)return;
      if((g.updated||'')>(prev.updated||''))byName[key]=g;
    });
    var list=[];for(var k in byName)list.push(byName[k]);
    list.sort(function(a,b){return (b.active?1:0)-(a.active?1:0)||(b.updated||'').localeCompare(a.updated||'')});
    grid.innerHTML='';
    if(!list.length){grid.innerHTML='<div class="empty">No local games yet. Use the canvas to create one.</div>';return}
    list.forEach(function(g){
      var card=makeCard(g.name,g.scope,g.version,g.size,g.active);
      card.addEventListener('click',function(){setActiveGame(g.id);goCanvas()});
      grid.appendChild(card);
    });
  })();

  // Relay games
  (async function loadRelay(){
    var grid=document.getElementById('relayGrid');
    try{
      var res=await fetch('https://relay.slob.games/sync/games');
      var games=await res.json();
      grid.innerHTML='';
      if(!games.length){grid.innerHTML='<div class="empty">No community games available.</div>';return}
      games.sort(function(a,b){return a.name.localeCompare(b.name)});
      games.forEach(function(g){
        var card=makeCard(g.name,'relay','',g.size,false);
        card.addEventListener('click',function(){fetchAndPlay(g.content_hash,g.name)});
        grid.appendChild(card);
      });
    }catch(e){
      grid.innerHTML='<div class="empty">Could not load community games.</div>';
    }
  })();

  async function fetchAndPlay(hash,name){
    try{
      var res=await fetch('https://relay.slob.games/sync/game/'+hash);
      var data=await res.json();
      if(!data.content)return;
      var col=readGamesCollection();
      var id='s-'+hash.substring(0,16);
      col.games[id]={name:name,content:data.content,scope:'external',version:'',created:new Date().toISOString(),updated:new Date().toISOString()};
      col.active=id;
      var pvfs=JSON.parse(localStorage.getItem('traits.pvfs')||'{}');
      pvfs['canvas/games.json']=JSON.stringify(col);
      localStorage.setItem('traits.pvfs',JSON.stringify(pvfs));
      goCanvas();
    }catch(e){console.error('Failed to load game:',e)}
  }
})();
"##;
