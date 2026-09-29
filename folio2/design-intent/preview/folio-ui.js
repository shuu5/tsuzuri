/* folio v2 chrome（自己完結・外部依存 0・head で読む 1 本）。図は HTML 部品なので JS 無効でも読める・全リンクが効く・説明の小窓は CSS だけで開く。
   ここで足すのは 3 つだけ: (1) 図の拡大 = ボタンで開く / ✕・Esc・背景で閉じる の 2 状態
   (2) 説明の小窓の位置補正（右端・下端で反転）・「外を押したら閉じる」・開いたら他を閉じる・狭幅シートの閉じるボタン
   (3) 文字の大きさ = 3 段（標準 / 大 / 特大）・localStorage に記憶・描画前に root へ適用。 */
(function(){
  var root=document.documentElement, KEY='folio2.fs', LV=['標準','大','特大'];
  root.classList.remove('no-js');
  function get(){ try{ var v=parseInt(localStorage.getItem(KEY),10); return (v>=0&&v<LV.length)?v:0; }catch(e){ return 0; } }
  function put(v){ try{ localStorage.setItem(KEY,String(v)); }catch(e){} }
  function apply(v){
    if(v>0) root.setAttribute('data-fs',String(v)); else root.removeAttribute('data-fs');
    document.querySelectorAll('.fs-btn').forEach(function(b){ var n=b.querySelector('.fs-now'); if(n) n.textContent=LV[v]; b.setAttribute('aria-label','文字の大きさを切り替える（いま: '+LV[v]+'）'); });
  }
  var lv=get(); if(lv>0) root.setAttribute('data-fs',String(lv));   /* 描画前に効かせる（ボタンの表示は DOM 後） */
  function ready(){
    apply(lv);
    var bd=null,cur=null, narrow=window.matchMedia('(max-width:680px)');
    function backdrop(){ if(!bd){ bd=document.createElement('div'); bd.className='fig-backdrop'; bd.hidden=true; document.body.appendChild(bd); bd.addEventListener('click',close); } return bd; }
    function close(){ if(!cur) return; cur.classList.remove('is-zoomed'); cur=null; backdrop().hidden=true; root.classList.remove('has-zoom'); }
    function open(f){ if(cur&&cur!==f) close(); cur=f; f.classList.add('is-zoomed'); backdrop().hidden=false; root.classList.add('has-zoom'); f.scrollTop=0; }
    /* 小窓: checkbox を外し focus も外す（:has(input:focus-visible) が残って開きっぱなしにならないように） */
    /* except を含む小窓の祖先（入れ子の外側）は閉じない。閉じた小窓には just-closed を付け、ポインタが離れるまで :hover で再表示しない */
    function closeOne(i){ var h=i.closest('.hint'); if(i.checked) i.checked=false; if(document.activeElement===i) i.blur(); if(h) h.classList.add('just-closed'); }
    function closeHints(except){ var keep=except?except.closest('.hint'):null; document.querySelectorAll('.hint input').forEach(function(i){ if(i===except) return; var h=i.closest('.hint'); if(keep&&h&&h.contains(keep)) return; if(i.checked||document.activeElement===i) closeOne(i); }); }
    /* 小窓が画面右端・下端からはみ出すなら反転（狭幅シートでは不要） */
    function fit(h){ var b=h.querySelector('.hint-body'); if(!b) return; h.classList.remove('flip','flip-up','flip-down'); if(narrow.matches) return; var r=b.getBoundingClientRect(); if(r.right>window.innerWidth-8) h.classList.add('flip'); if(r.bottom>window.innerHeight-8&&r.height<r.top) h.classList.add('flip-up'); if(r.top<8) h.classList.add('flip-down'); }
    document.querySelectorAll('.hint').forEach(function(h){
      var i=h.querySelector('input'), b=h.querySelector('.hint-body'); if(!i||!b) return;
      if(!b.querySelector('.hint-close')){ var x=document.createElement('button'); x.type='button'; x.className='hint-close'; x.setAttribute('aria-label','閉じる'); x.textContent='閉じる ✕'; b.insertBefore(x,b.firstChild); }
      h.addEventListener('mouseenter',function(){ fit(h); });
      h.addEventListener('mouseleave',function(){ h.classList.remove('just-closed'); });
      i.addEventListener('change',function(){ if(i.checked){ closeHints(i); fit(h); } else { h.classList.add('just-closed'); } });
    });
    document.addEventListener('click',function(e){
      var fb=e.target.closest('.fs-btn'); if(fb){ lv=(lv+1)%LV.length; put(lv); apply(lv); return; }
      var hc=e.target.closest('.hint-close'); if(hc){ var hi=hc.closest('.hint').querySelector(':scope>label>input'); if(hi) closeOne(hi); return; }   /* 閉じるは自分の小窓だけ（入れ子の外側は残す） */
      var zb=e.target.closest('.zoom-btn'); if(zb){ e.preventDefault(); open(zb.closest('[data-role="diagram"]')); return; }
      if(e.target.closest('.zoom-close')){ e.preventDefault(); close(); return; }
      if(cur && e.target.closest('a[href]') && cur.contains(e.target)) close();      /* 拡大中のリンクは先に閉じてから遷移 */
      if(!e.target.closest('.hint')) closeHints();                                   /* 小窓は外を押したら閉じる */
    });
    document.addEventListener('keydown',function(e){ if(e.key==='Escape'){ close(); closeHints(); } });
    window.addEventListener('hashchange',close);
    /* 印刷: 畳んだ details を開いて紙に出し、終わったら戻す（CSS の ::details-content と二重化） */
    window.addEventListener('beforeprint',function(){ document.querySelectorAll('details:not([open])').forEach(function(d){ d.open=true; d.setAttribute('data-print-opened',''); }); });
    window.addEventListener('afterprint',function(){ document.querySelectorAll('details[data-print-opened]').forEach(function(d){ d.open=false; d.removeAttribute('data-print-opened'); }); });
    window.addEventListener('storage',function(e){ if(e.key===KEY){ lv=get(); apply(lv); } });   /* 別タブで変えた大きさに追随 */
  }
  if(document.readyState==='loading') document.addEventListener('DOMContentLoaded',ready); else ready();
})();
