<template>
  <div ref="shellRef" class="app-shell" :class="[{ dark: settings.dark }, `${mode}-mode`]" :style="shellStyle" @contextmenu.prevent @pointermove="noteActivity" @pointerdown="noteActivity" @keydown="noteActivity">
    <div v-if="mode === 'ball'" class="ball-wrap mode-surface">
      <button class="floating-ball" :class="{ 'image-ball': settings.ballStyle === 'image', 'text-ball': settings.ballStyle === 'text', 'crop-circle': settings.ballImageCrop }" :style="ballStyle" :aria-label="t('settings')" @contextmenu.prevent @pointerdown.prevent="onBallDown" @pointermove.prevent="onBallMove" @pointerup.prevent="onBallUp" @pointercancel.prevent="onBallCancel">
        <img v-if="settings.ballStyle === 'image'" :src="settings.ballImage" alt="" />
        <span v-else-if="settings.ballStyle === 'text'" class="ball-text"><span v-for="line in ballTextLines" :key="line">{{ line }}</span></span>
        <Icon v-else icon="fluent:compass-northwest-24-filled" />
      </button>
    </div>

    <div v-else-if="mode === 'compass'" class="compass-view mode-surface">
      <header class="compass-titlebar" @pointerdown="onTitlebarPointerDown" @pointermove="onTitlebarPointerMove" @pointerup="onTitlebarPointerUp" @pointercancel="onTitlebarPointerCancel">
        <div class="brand"><img src="/cyrene.png" alt="" /><strong>Cyreneの罗盘</strong></div>
        <button class="icon-btn" :title="t('settings')" @click="openSettings"><Icon icon="fluent:settings-24-regular" /></button>
      </header>
      <main class="compass-stage">
        <div ref="gridRef" class="compass-grid">
          <button v-for="item in compassItems" :key="item.id" class="compass-item" :class="{ empty: !item.label }" @click="selectItem(item)">
            <img v-if="item.iconMode === 'custom' && item.customIcon" class="system-icon custom-item-icon" :src="item.customIcon" alt="" />
            <img v-else-if="item.iconMode !== 'iconify' && item.systemIcon" class="system-icon" :src="item.systemIcon" alt="" />
            <Icon v-else class="item-icon" :icon="item.icon || 'fluent:app-generic-24-regular'" />
            <span class="item-label">{{ item.label }}</span><Icon v-if="item.children?.length" class="item-chevron" icon="fluent:chevron-right-16-regular" />
          </button>
          <button class="compass-center" @pointerdown.prevent="onCenterPointerDown" @pointermove.prevent="onCenterPointerMove" @pointerup.prevent="onCenterPointerUp" @pointercancel.prevent="onCenterPointerCancel"><Icon :icon="breadcrumb.length ? 'fluent:arrow-left-24-regular' : 'fluent:dismiss-24-regular'" /></button>
        </div>
      </main>
      <span class="version-badge">v{{ version }}</span>
    </div>

    <div v-else class="settings-view mode-surface">
      <header class="titlebar" @pointerdown="onTitlebarPointerDown" @pointermove="onTitlebarPointerMove" @pointerup="onTitlebarPointerUp" @pointercancel="onTitlebarPointerCancel"><div class="brand"><img src="/cyrene.png" alt="" /><strong>Cyreneの罗盘</strong></div><button class="icon-btn close-window-btn" :title="t('backToBall')" @click="returnToBall"><Icon icon="fluent:dismiss-24-regular" /></button></header>
      <div class="settings-layout">
        <aside class="dock"><button v-for="item in dockItems" :key="item.id" class="dock-item" :class="{ active: section === item.id }" @click="section = item.id"><Icon :icon="item.icon" /><label>{{ t(item.label) }}</label></button></aside>
        <main class="settings-content">
          <Transition :css="false" mode="out-in" @enter="onPageEnter" @leave="onPageLeave">
          <section v-if="section === 'general'" key="general">
            <h1>{{ t('general') }}</h1><p class="lead">{{ t('generalLead') }}</p>
            <div class="setting-card"><h2>{{ t('theme') }}</h2><div class="segmented"><button :class="{ active: settings.theme === 'peach' }" @click="setTheme('peach')">{{ t('peach') }}</button><button :class="{ active: settings.theme === 'fluent' }" @click="setTheme('fluent')">Fluent</button><button :class="{ active: settings.theme === 'custom' }" @click="setTheme('custom')">{{ t('custom') }}</button></div><label class="field"><span>{{ t('language') }}</span><select v-model="settings.language" @change="persist"><option value="zh">简体中文</option><option value="en">English</option></select></label><label v-if="settings.theme === 'custom'" class="field color-field"><span>{{ t('accent') }}</span><span><input v-model="settings.accent" type="color" @change="persist" /><code>{{ settings.accent }}</code></span></label><div class="field"><span>{{ t('dark') }}</span><FluentSwitch v-model="settings.dark" @change="persist" /></div></div>
            <div class="setting-card"><h2>{{ t('appearance') }}</h2><label class="field"><span>{{ t('ballStyle') }}</span><select v-model="settings.ballStyle" @change="persist"><option value="text">{{ t('textBall') }}</option><option value="solid">{{ t('iconBall') }}</option><option value="image">{{ t('image') }}</option></select></label><template v-if="settings.ballStyle !== 'image'"><label class="field color-field"><span>{{ t('ballColor') }}</span><input v-model="settings.ballColor" type="color" @change="persist" /></label><label v-if="settings.ballStyle === 'text'" class="field"><span>{{ t('ballText') }}</span><input class="ball-text-input" :value="settings.ballText" :placeholder="t('ballTextPlaceholder')" maxlength="4" @input="setBallText" @change="persist" /></label></template><template v-else><label class="field"><span>{{ t('chooseBallImage') }}</span><button class="secondary-btn" @click="chooseBallImage"><Icon icon="fluent:image-add-24-regular" />{{ t('chooseBallImage') }}</button></label><div class="field"><span>{{ t('cropBallImage') }}</span><FluentSwitch v-model="settings.ballImageCrop" @change="persist" /></div></template><NumberField v-model="settings.ballSize" :label="t('ballSize')" unit="px" :min="44" :max="160" @change="persist"/><NumberField v-model="settings.ballOpacity" :label="t('ballOpacity')" unit="%" :min="25" :max="100" @change="persist"/><NumberField v-model="settings.compassOpacity" :label="t('compassOpacity')" unit="%" :min="55" :max="100" @change="persist"/><NumberField v-model="settings.compassSize" :label="t('compassSize')" unit="px" :min="420" :max="820" :step="10" @change="persist"/><div class="field scale-field"><span>{{ t('uiScale') }}</span><div class="scale-apply"><FluentNumberInput v-model="uiScaleDraft" unit="%" :min="50" :max="150" :step="5"/><button class="primary-btn compact-btn" @click="applyUiScale">{{ t('apply') }}</button></div></div><NumberField v-model="settings.iconSize" :label="t('iconSize')" unit="px" :min="24" :max="72" @change="persist"/><NumberField v-model="settings.fontSize" :label="t('fontSize')" unit="px" :min="11" :max="26" @change="persist"/><label class="field"><span>{{ t('fontFamily') }}</span><select v-model="settings.fontFamily" @change="persist"><option value="HarmonyOS">HarmonyOS Sans SC</option><option value="Segoe UI Variable">Segoe UI Variable</option><option value="Microsoft YaHei UI">Microsoft YaHei UI</option></select></label></div>
            <div class="setting-card"><h2>{{ t('behavior') }}</h2><div class="field"><span>{{ t('autoHide') }}</span><FluentSwitch v-model="settings.autoHide" @change="persist" /></div><NumberField v-model="settings.autoHideSeconds" :label="t('autoHideSeconds')" :unit="t('seconds')" :min="3" :max="30" @change="persist"/></div>
            <div class="setting-card"><h2>{{ t('startup') }}</h2><div class="field"><span>{{ t('scheduled') }}</span><FluentSwitch v-model="settings.autoStart" @change="toggleStartup('scheduled')" /></div><div class="field"><span>{{ t('registry') }}</span><FluentSwitch v-model="settings.registryStart" @change="toggleStartup('registry')" /></div></div>
          </section>
          <section v-else-if="section === 'actions'" key="actions">
            <h1>{{ t('actions') }}</h1><p class="lead">{{ t('actionsLead') }}</p>
            <button class="drop-zone" @click="browsePath"><Icon icon="fluent:document-arrow-up-24-regular" /><strong>{{ t('dropTitle') }}</strong><small>{{ t('dropHint') }}</small></button>
            <div v-if="editBreadcrumb.length" class="edit-breadcrumb"><button @click="goEditLevel(-1)"><Icon icon="fluent:arrow-left-20-regular" />{{ t('actions') }}</button><template v-for="(menu, level) in editBreadcrumb" :key="menu.id"><span>/</span><button @click="goEditLevel(level)">{{ menu.label }}</button></template></div>
            <div class="action-list"><article v-for="(item, index) in currentEditableItems" :key="item.id" class="action-row" :class="{ 'submenu-row': item.children }">
              <button class="preview-icon" :title="t('chooseIcon')" @click="openIconPicker(item)"><img v-if="item.iconMode === 'custom' && item.customIcon" :src="item.customIcon" alt="" /><img v-else-if="item.iconMode !== 'iconify' && item.systemIcon" :src="item.systemIcon" alt="" /><Icon v-else :icon="item.icon || 'fluent:app-generic-24-regular'" /></button>
              <div class="action-main"><input v-model="item.label" :placeholder="t('label')" @change="persist" /><input v-if="!item.children" v-model="item.target" :placeholder="t('target')" @change="persist" /><small>{{ t('type') }}：{{ itemTypeLabel(item) }}</small></div>
              <button v-if="item.children" class="secondary-btn enter-submenu" @click="editBreadcrumb.push(item)"><Icon icon="fluent:folder-open-24-regular" />{{ t('enterSubmenu') }}</button><div v-else class="admin-check"><span>{{ t('administrator') }}</span><FluentSwitch v-model="item.elevated" @change="persist" /></div><button class="icon-btn danger" :title="t('clear')" @click="clearItem(index)"><Icon icon="fluent:delete-24-regular" /></button>
            </article></div>
            <div class="button-row"><button class="primary-btn" @click="addAction(false)"><Icon icon="fluent:add-20-regular" />{{ t('addAction') }}</button><button class="secondary-btn" @click="addAction(true)"><Icon icon="fluent:folder-add-20-regular" />{{ t('addSubmenu') }}</button></div>
          </section>
          <section v-else key="about">
            <h1>{{ t('about') }}</h1>
            <div class="about-hero">
              <img src="/cyrene.png" alt="Cyrene" />
              <div><h2>Cyreneの罗盘</h2><p>{{ t('aboutLead') }}</p><p class="muted">{{ t('version') }} {{ version }} · {{ t(BUILD_VARIANT === 'uiaccess' ? 'uiAccessEdition' : 'standardEdition') }}</p></div>
            </div>
            <div class="about-details">
              <div><span>{{ t('author') }}</span><strong>Cyrene2008 (星海昔涟)</strong></div>
              <div><span>{{ t('copyright') }}</span><strong>Copyright (C) Cyrene2008 2026. All Rights Reserved.</strong></div>
              <div><span>{{ t('license') }}</span><button class="inline-link" @click="openExternal('https://www.gnu.org/licenses/gpl-3.0.html')">GNU GPLv3<Icon icon="fluent:open-16-regular" /></button></div>
            </div>
            <div class="about-actions"><button class="secondary-btn" @click="openExternal('https://github.com/Cyrene2008/CyreneCompass')"><Icon icon="fluent:code-24-regular" />{{ t('openRepository') }}</button></div>
            <div class="update-section">
              <div class="update-heading"><div><h2>{{ t('softwareUpdate') }}</h2><p>{{ updateStatusText }}</p></div><button class="secondary-btn" :disabled="updateState.checking || updateState.downloading" @click="checkForUpdates(false)"><Icon :icon="updateState.checking ? 'fluent:arrow-sync-24-regular' : 'fluent:arrow-clockwise-24-regular'" :class="{ spinning: updateState.checking }" />{{ updateState.checking ? t('checkingUpdate') : t('checkUpdate') }}</button></div>
              <template v-if="updateState.available">
                <div class="update-release"><strong>{{ t('newVersion') }} {{ updateState.version }}</strong><button v-if="updateState.releaseUrl" class="inline-link" @click="openExternal(updateState.releaseUrl)">{{ t('releaseNotes') }}<Icon icon="fluent:open-16-regular" /></button></div>
                <button class="primary-btn" :disabled="updateState.downloading || !updateState.url" @click="downloadUpdate"><Icon icon="fluent:arrow-download-24-regular" />{{ updateState.downloading ? t('installingUpdate') : t('downloadInstall') }}</button>
              </template>
              <div v-if="updateState.downloading" class="update-progress" role="progressbar" :aria-valuenow="Math.round(updateState.progress)" aria-valuemin="0" aria-valuemax="100"><span :style="{ width: `${updateState.progress}%` }"></span></div>
              <p v-if="updateState.downloading" class="update-progress-text">{{ t('downloadProgress') }} {{ Math.round(updateState.progress) }}%</p>
              <p v-if="updateState.errorKey || updateState.error" class="update-error">{{ t('updateFailed') }}：{{ updateState.errorKey ? t(updateState.errorKey) : updateState.error }}</p>
            </div>
            <button class="danger-btn" @click="beginExit"><Icon icon="fluent:power-24-regular" />{{ t('quit') }}</button>
          </section>
          </Transition>
        </main>
      </div>
    </div>

    <div v-if="iconPickerItem" class="modal-layer"><div class="modal"><h2>{{ t('iconLibrary') }}</h2><input v-model="iconSearch" class="search" :placeholder="t('searchIcon')" /><div class="icon-grid"><button class="custom-icon-choice" :title="t('customIcon')" @click="chooseCustomItemIcon"><Icon icon="fluent:image-add-24-regular" /><span>{{ t('custom') }}</span></button><button v-for="name in filteredIcons" :key="name" :title="name" @click="chooseIcon(name)"><Icon :icon="name" /></button></div><div class="modal-actions"><button class="secondary-btn" @click="restoreSystemIcon">{{ t('useSystemIcon') }}</button><button class="primary-btn" @click="iconPickerItem = null">{{ t('close') }}</button></div></div></div>
    <div v-if="exitStage" class="modal-layer"><div class="modal confirm-modal"><Icon class="confirm-icon" icon="fluent:warning-24-filled" /><h2>{{ exitText }}</h2><div class="modal-actions"><button class="secondary-btn" @click="exitSecondary">{{ exitStage === 3 ? t('confirm') : t('cancel') }}</button><button class="primary-btn" @click="exitPrimary">{{ exitStage === 3 ? t('cancel') : t('confirm') }}</button></div></div></div>
    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { Icon, addCollection } from '@iconify/vue'
import fluentIcons from './fluent-icons.json'
import { gsap } from 'gsap'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, PhysicalPosition } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { createThemeVariables, DEFAULT_ACCENT } from './theme'
import { translate } from './i18n'
import FluentSwitch from './components/FluentSwitch.vue'
import FluentNumberInput from './components/FluentNumberInput.vue'
import NumberField from './components/SettingNumberField.vue'
import { clientOffsetToPhysical, createPointerMoveState, exceedsDragThreshold, mouseDragTarget, touchDragTarget } from './utils/physicalDrag'
import { BUILD_VARIANT, CURRENT_VERSION, checkForUpdates, downloadUpdate, updateState } from './updater'

addCollection(fluentIcons)
const version = CURRENT_VERSION; const mode = ref('ball'); const section = ref('general'); const breadcrumb = ref([]); const editBreadcrumb = ref([]); const toast = ref(''); const gridRef = ref(); const shellRef = ref(); const iconPickerItem = ref(null); const iconSearch = ref(''); const exitStage = ref(0); const systemAccent = ref('#0078d4'); let idleTimer; let ballPointer; let centerPointer; let titlebarPointer; let transitioning = false; const cleanups = []
const iconNames = Object.keys(fluentIcons.icons).map(name => `fluent:${name}`)
const baseItems = () => [{ id:'1',label:'演示文稿',icon:'fluent:slide-text-24-regular',target:'powerpnt.exe'},{id:'2',label:'浏览器',icon:'fluent:globe-24-regular',target:'https://www.bing.com'},{id:'3',label:'文件',icon:'fluent:folder-24-regular',target:'explorer.exe'},{id:'4',label:'截图',icon:'fluent:screenshot-24-regular',target:'snippingtool.exe'},{id:'5',label:'进程查看',icon:'fluent:code-24-regular',target:'Get-Process | Select-Object -First 5',kind:'script'},{id:'6',label:'终端',icon:'fluent:window-dev-tools-24-regular',target:'wt.exe'},{id:'7',label:'计算器',icon:'fluent:calculator-24-regular',target:'calc.exe'},{id:'8',label:'教学工具',icon:'fluent:toolbox-24-regular',children:[{id:'8-1',label:'记事本',icon:'fluent:notepad-24-regular',target:'notepad.exe'}]}]
let saved = null; try { saved = JSON.parse(localStorage.getItem('cyrene-compass-settings') || 'null') } catch {}
function migrateLegacyDefaults(value){if(!value?.items)return value;const replacements=new Map([['1',['PowerPoint','fluent:slide-text-24-regular','powerpnt.exe','演示文稿','fluent:slide-text-24-regular','powerpnt.exe']],['2',['Browser','fluent:browser-24-regular','https://www.bing.com','浏览器','fluent:globe-24-regular','https://www.bing.com']],['3',['Files','fluent:folder-24-regular','','文件','fluent:folder-24-regular','explorer.exe']],['4',['Screenshot','fluent:camera-24-regular','snippingtool.exe','截图','fluent:screenshot-24-regular','snippingtool.exe']],['5',['PowerShell','fluent:code-24-regular','Get-Process | Select-Object -First 5','进程查看','fluent:code-24-regular','Get-Process | Select-Object -First 5']],['6',['Terminal','fluent:terminal-24-regular','wt.exe','终端','fluent:window-dev-tools-24-regular','wt.exe']],['7',['Calculator','fluent:calculator-24-regular','calc.exe','计算器','fluent:calculator-24-regular','calc.exe']],['8',['Tools','fluent:apps-24-regular','', '教学工具','fluent:toolbox-24-regular','']],['8-1',['Notepad','fluent:notepad-24-regular','notepad.exe','记事本','fluent:notepad-24-regular','notepad.exe']]]);const visit=items=>items?.forEach(item=>{const rule=replacements.get(item.id);const oldTarget=item.target??'';if(rule&&item.label===rule[0]&&item.icon===rule[1]&&oldTarget===rule[2]){item.label=rule[3];item.icon=rule[4];if(!item.children)item.target=rule[5]}visit(item.children)});visit(value.items);return value}
saved=migrateLegacyDefaults(saved)
if (saved && saved.settingsVersion !== 4) {const usesLegacyDefaultBall=(saved.ballStyle??'solid')==='solid'&&(saved.ballColor??DEFAULT_ACCENT).toLowerCase()===DEFAULT_ACCENT; saved = { ...saved, settingsVersion: 4, uiScale: saved.uiScale ?? 100, iconSize: saved.iconSize ?? 42, fontSize: saved.fontSize ?? 15, ballImageCrop: saved.ballImageCrop ?? false, ballStyle: usesLegacyDefaultBall ? 'text' : (saved.ballStyle ?? 'text'), ballText: saved.ballText ?? '罗盘' }}
const settings = ref({ settingsVersion:4,language:'zh',theme:'peach',accent:DEFAULT_ACCENT,dark:false,ballStyle:'text',ballText:'罗盘',ballColor:DEFAULT_ACCENT,ballSize:72,ballOpacity:86,compassOpacity:100,compassSize:560,uiScale:100,iconSize:42,fontSize:15,fontFamily:'HarmonyOS',ballImage:'/cyrene.png',ballImageCrop:false,autoHide:true,autoHideSeconds:10,autoStart:false,registryStart:false,items:baseItems(),...saved })
const uiScaleDraft = ref(settings.value.uiScale)
const editableItems = ref(settings.value.items.map(x => ({...x})))
const currentEditableItems = computed(() => editBreadcrumb.value.length ? editBreadcrumb.value.at(-1).children : editableItems.value)
const t = key => translate(settings.value.language, key)
const dockItems = [{id:'general',icon:'fluent:settings-24-regular',label:'general'},{id:'actions',icon:'fluent:grid-24-regular',label:'actions'},{id:'about',icon:'fluent:info-24-regular',label:'about'}]
const compassItems = computed(() => Array.from({length:8},(_,i)=>(breadcrumb.value.length ? breadcrumb.value.at(-1).children : editableItems.value)[i] || {id:`empty-${i}`,label:''}))
const filteredIcons = computed(() => iconNames.filter(x => x.includes(iconSearch.value.toLowerCase())))
const ballTextLines = computed(() => {const chars=Array.from(settings.value.ballText || '罗盘').slice(0,4);return chars.length<=2?[chars.join('')]:[chars.slice(0,2).join(''),chars.slice(2).join('')]})
const shellStyle = computed(() => { const scale = mode.value === 'ball' ? 1 : settings.value.uiScale / 100; const accent = settings.value.theme === 'fluent' ? systemAccent.value : settings.value.theme === 'peach' ? DEFAULT_ACCENT : settings.value.accent; return {...createThemeVariables(accent, settings.value.dark, settings.value.theme === 'fluent'),'--ui-scale':scale,'--item-min':`${32/scale}px`,'--icon-size':`${settings.value.iconSize}px`,'--item-font-size':`${settings.value.fontSize}px`,'--font-ui':`'${settings.value.fontFamily}', 'HarmonyOS', 'Segoe UI Variable', sans-serif`,'--panel-alpha':settings.value.compassOpacity/100} })
const ballStyle = computed(() => ({width:`${settings.value.ballSize}px`,height:`${settings.value.ballSize}px`,opacity:settings.value.ballOpacity/100,background:settings.value.ballStyle==='image'?'transparent':settings.value.ballColor,'--ball-text-size':`${Math.max(12,settings.value.ballSize*(ballTextLines.value.length>1?.235:.285))}px`}))
const exitText = computed(() => t(exitStage.value === 1 ? 'quit1' : exitStage.value === 2 ? 'quit2' : 'quit3'))
const updateStatusText = computed(() => {if(updateState.value.checking)return t('checkingUpdate');if(updateState.value.available)return `${t('updateAvailable')} ${updateState.value.version}`;if(updateState.value.checked&&!updateState.value.errorKey&&!updateState.value.error)return t('latestVersion');return t('updateLead')})
function persist(){ settings.value.items=editableItems.value; localStorage.setItem('cyrene-compass-settings',JSON.stringify(settings.value)); noteActivity() }
async function setTheme(v){settings.value.theme=v;if(v==='peach')settings.value.accent=DEFAULT_ACCENT;if(v==='fluent')await refreshSystemAccent();persist()}
function showToast(v){toast.value=v;setTimeout(()=>toast.value='',2200)}
async function resizeWindow(next){if(transitioning||next===mode.value)return;transitioning=true;breadcrumb.value=[];clearTimeout(idleTimer);try{const currentSurface=shellRef.value?.querySelector('.mode-surface');if(currentSurface)await gsap.to(currentSurface,{opacity:0,scale:.985,duration:.05,ease:'power1.in'});const scale=next==='ball'?1:settings.value.uiScale/100;const baseSize=next==='compass'?settings.value.compassSize:next==='settings'?1120:Math.max(88,settings.value.ballSize+16);const width=baseSize*scale;const height=(next==='compass'?settings.value.compassSize+70:next==='settings'?760:baseSize)*scale;mode.value=next;await nextTick();gsap.set(shellRef.value?.querySelector('.mode-surface'),{opacity:0,scale:.98});try{await invoke('set_window_mode',{mode:next,opacity:1,width,height,animate:true})}catch{}const nextSurface=shellRef.value?.querySelector('.mode-surface');if(nextSurface)await gsap.fromTo(nextSurface,{opacity:0,scale:.98},{opacity:1,scale:1,duration:.1,ease:'power2.out'});if(next==='compass')noteActivity()}finally{transitioning=false}}
function openCompass(){return resizeWindow('compass')}
function openSettings(){resizeWindow('settings')}; function returnToBall(){resizeWindow('ball')}
function pointerSample(e){const events=typeof e.getCoalescedEvents==='function'?e.getCoalescedEvents():null;const latest=events?.length?events[events.length-1]:e;return{clientX:latest.clientX,clientY:latest.clientY,screenX:latest.screenX,screenY:latest.screenY}}
function thresholdPoint(pointer,sample){return pointer.pointerType==='mouse'?{x:sample.screenX,y:sample.screenY}:{x:sample.clientX,y:sample.clientY}}
function makePointer(e,source){const sample=pointerSample(e);return createPointerMoveState({id:e.pointerId,pointerType:e.pointerType,source,start:thresholdPoint({pointerType:e.pointerType},sample),startClientX:e.clientX,startClientY:e.clientY,lastScreenX:e.screenX,lastScreenY:e.screenY,dragged:false,active:true})}
async function saveDraggedPosition(source){const win=getCurrentWindow();const pos=await win.outerPosition();if(source!=='ball'){const size=await win.outerSize();const centerX=Math.round(pos.x+size.width/2);const centerY=Math.round(pos.y+size.height/2);await invoke('set_ball_anchor',{x:centerX,y:centerY});const factor=await win.scaleFactor();const ballSize=Math.round(Math.max(88,settings.value.ballSize+16)*factor);settings.value.ballPosition={x:Math.round(centerX-ballSize/2),y:Math.round(centerY-ballSize/2)}}else{settings.value.ballPosition={x:pos.x,y:pos.y}}persist()}
async function startManualDrag(pointer){const win=getCurrentWindow();const initial=await win.outerPosition();const factor=await win.scaleFactor();const grabOffset=clientOffsetToPhysical(pointer.startClientX,pointer.startClientY,factor);let mousePosition={x:initial.x,y:initial.y};return{move:async sample=>{if(pointer.pointerType==='mouse'){const target=mouseDragTarget(mousePosition,{x:pointer.lastScreenX,y:pointer.lastScreenY},{x:sample.screenX,y:sample.screenY},factor);pointer.lastScreenX=sample.screenX;pointer.lastScreenY=sample.screenY;if(target.x===mousePosition.x&&target.y===mousePosition.y)return;await win.setPosition(new PhysicalPosition(target.x,target.y));mousePosition=target;return}const current=await win.outerPosition();const target=touchDragTarget(current,{x:sample.clientX,y:sample.clientY},factor,grabOffset);if(Math.abs(target.x-current.x)<=1&&Math.abs(target.y-current.y)<=1)return;await win.setPosition(new PhysicalPosition(target.x,target.y))},end:()=>saveDraggedPosition(pointer.source)}}
function enqueuePointerSample(pointer,e){pointer.queue.push(pointerSample(e))}
async function finishLostMousePointer(pointer,e){pointer.active=false;if(e.currentTarget.hasPointerCapture(e.pointerId))e.currentTarget.releasePointerCapture(e.pointerId);if(pointer.dragged){await pointer.queue.flush();const drag=await pointer.drag;await drag.end()}else pointer.queue.clear()}
function pointerMove(pointer,e,allowDrag=true){if(!pointer||!pointer.active||pointer.id!==e.pointerId)return pointer;if(e.pointerType==='mouse'&&(e.buttons&1)===0){finishLostMousePointer(pointer,e).catch(()=>{});return null}const sample=pointerSample(e);if(!pointer.dragged&&allowDrag&&exceedsDragThreshold(pointer.start,thresholdPoint(pointer,sample))){pointer.dragged=true;pointer.drag=startManualDrag(pointer)}if(pointer.dragged)enqueuePointerSample(pointer,e);return pointer}
async function finishPointer(pointer,e,cancelled=false){if(!pointer||pointer.id!==e.pointerId)return{clicked:false};const clicked=!pointer.dragged&&!cancelled;if(pointer.dragged){enqueuePointerSample(pointer,e);await pointer.queue.flush();const drag=await pointer.drag;await drag.end()}pointer.active=false;pointer.queue.clear();if(e.currentTarget.hasPointerCapture(e.pointerId))e.currentTarget.releasePointerCapture(e.pointerId);return{clicked}}
function onCenterPointerDown(e){if(e.pointerType==='mouse'&&e.button!==0)return;e.currentTarget.setPointerCapture(e.pointerId);centerPointer=makePointer(e,'center')}
function onCenterPointerMove(e){centerPointer=pointerMove(centerPointer,e,!breadcrumb.value.length)}
async function animateCompassPage(direction, change){const items=gridRef.value?.querySelectorAll('.compass-item');if(items?.length)await gsap.to(items,{opacity:0,x:direction*16,duration:.12,stagger:.008,ease:'power2.in'});change();await nextTick();const nextItems=gridRef.value?.querySelectorAll('.compass-item');if(nextItems?.length)gsap.fromTo(nextItems,{opacity:0,x:-direction*18},{opacity:1,x:0,duration:.24,stagger:.012,ease:'power3.out'})}
async function onCenterPointerUp(e){const pointer=centerPointer;centerPointer=null;const result=await finishPointer(pointer,e);if(result.clicked){if(breadcrumb.value.length)await animateCompassPage(-1,()=>breadcrumb.value.pop());else returnToBall()}}
async function onCenterPointerCancel(e){const pointer=centerPointer;centerPointer=null;await finishPointer(pointer,e,true)}
function onTitlebarPointerDown(e){if(e.target.closest('button,input,select,textarea,a'))return;if(e.pointerType==='mouse'&&e.button!==0)return;e.preventDefault();e.currentTarget.setPointerCapture(e.pointerId);titlebarPointer=makePointer(e,'titlebar')}
function onTitlebarPointerMove(e){titlebarPointer=pointerMove(titlebarPointer,e,true)}
async function onTitlebarPointerUp(e){const pointer=titlebarPointer;titlebarPointer=null;await finishPointer(pointer,e)}
async function onTitlebarPointerCancel(e){const pointer=titlebarPointer;titlebarPointer=null;await finishPointer(pointer,e,true)}
async function selectItem(item){if(!item.label)return;if(item.children?.length){await animateCompassPage(1,()=>breadcrumb.value.push(item));noteActivity();return}try{await invoke('execute_action',{target:item.target||'',elevated:!!item.elevated,script:item.kind==='script'});setTimeout(returnToBall,100)}catch(e){showToast(String(e))}}
function noteActivity(){clearTimeout(idleTimer);idleTimer=undefined;if(mode.value!=='compass'||!settings.value.autoHide)return;idleTimer=setTimeout(()=>{if(mode.value==='compass')returnToBall()},Math.max(3,Math.min(30,settings.value.autoHideSeconds))*1000)}
async function ingestPaths(paths){for(const path of paths){const list=currentEditableItems.value;if(list.length>=8&&!list.some(x=>!x.label)){showToast(t('menuFull'));break}try{const meta=await invoke('inspect_path',{path});const empty=list.findIndex(x=>!x.label);const item={id:crypto.randomUUID(),label:meta.label,target:meta.target||meta.path,resolvedTarget:meta.resolved_target,sourcePath:meta.path,kind:meta.kind,systemIcon:meta.icon_data_url,iconMode:'system',icon:'fluent:app-generic-24-regular',elevated:false};if(empty>=0)list[empty]=item;else list.push(item)}catch(e){showToast(`${t('inspectFailed')}: ${e}`)}}persist()}
async function browsePath(){const paths=await open({multiple:true,directory:false});if(paths)ingestPaths(Array.isArray(paths)?paths:[paths])}
async function chooseBallImage(){const path=await open({multiple:false,filters:[{name:'Images',extensions:['png','jpg','jpeg','webp','gif','bmp']} ]});if(!path)return;try{settings.value.ballImage=await invoke('read_visual_data_url',{path});persist()}catch(e){showToast(String(e))}}
function setBallText(e){settings.value.ballText=Array.from(e.target.value).slice(0,4).join('');e.target.value=settings.value.ballText}
function addAction(menu){if(currentEditableItems.value.length>=8){showToast(t('menuFull'));return}currentEditableItems.value.push({id:crypto.randomUUID(),label:t(menu?'newMenu':'emptyAction'),icon:'fluent:app-generic-24-regular',iconMode:'iconify',target:'',kind:menu?'submenu':'action',children:menu?[]:undefined,elevated:false});persist()}
function clearItem(i){currentEditableItems.value.splice(i,1);persist()};function openIconPicker(item){iconPickerItem.value=item;iconSearch.value=''};function chooseIcon(name){iconPickerItem.value.icon=name;iconPickerItem.value.iconMode='iconify';persist();iconPickerItem.value=null};async function chooseCustomItemIcon(){const path=await open({multiple:false,filters:[{name:'Images',extensions:['png','jpg','jpeg','webp','gif','bmp']} ]});if(!path)return;try{iconPickerItem.value.customIcon=await invoke('read_visual_data_url',{path});iconPickerItem.value.iconMode='custom';persist();iconPickerItem.value=null}catch(e){showToast(String(e))}}function restoreSystemIcon(){iconPickerItem.value.iconMode='system';persist();iconPickerItem.value=null}
async function toggleStartup(modeName){persist();try{await invoke('configure_startup',{enabled:modeName==='scheduled'?settings.value.autoStart:settings.value.registryStart,mode:modeName});showToast(t('startupUpdated'))}catch(e){showToast(String(e))}}
function beginExit(){exitStage.value=1};function exitSecondary(){if(exitStage.value===1){exitStage.value=0}else if(exitStage.value===2){exitStage.value=3}else{invoke('quit_app').catch(()=>window.close())}}function exitPrimary(){if(exitStage.value===1){exitStage.value=2}else{exitStage.value=0}}
function onBallDown(e){if(e.pointerType==='mouse'&&e.button!==0)return;e.currentTarget.setPointerCapture(e.pointerId);ballPointer=makePointer(e,'ball');ballPointer.startedAt=performance.now()}
function onBallMove(e){ballPointer=pointerMove(ballPointer,e,true)}
async function onBallUp(e){const pointer=ballPointer;ballPointer=null;const elapsed=pointer?performance.now()-pointer.startedAt:Infinity;const result=await finishPointer(pointer,e);if(result.clicked&&elapsed<650)openCompass()}
async function onBallCancel(e){const pointer=ballPointer;ballPointer=null;await finishPointer(pointer,e,true)}
function goEditLevel(level){editBreadcrumb.value=level<0?[]:editBreadcrumb.value.slice(0,level+1)}
function itemTypeLabel(item){if(item.children)return t('submenuType');if(item.kind==='script')return t('scriptType');if(item.kind==='shortcut')return t('shortcutType');if(item.kind==='folder')return t('folderType');if(item.kind&&item.kind!=='action'&&item.kind!=='file')return item.kind;return item.kind==='file'?t('fileType'):t('actionType')}
function onPageEnter(el,done){gsap.fromTo(el,{opacity:0,x:18},{opacity:1,x:0,duration:.28,ease:'power3.out',onComplete:done})}
function onPageLeave(el,done){gsap.to(el,{opacity:0,x:-12,duration:.16,ease:'power2.in',onComplete:done})}
async function refreshSystemAccent(){try{systemAccent.value=await invoke('system_accent')}catch{systemAccent.value='#0078d4'}}
async function openExternal(url){try{await invoke('open_external',{url})}catch(e){showToast(String(e))}}
async function applyUiScale(){settings.value.uiScale=uiScaleDraft.value;persist();if(mode.value==='settings'){const scale=settings.value.uiScale/100;try{await invoke('set_window_mode',{mode:'settings',opacity:1,width:1120*scale,height:760*scale,animate:true})}catch{}}}
function preventGlobalShortcuts(e){if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='a')e.preventDefault()}
function preventDrag(e){e.preventDefault()}
onMounted(async()=>{document.addEventListener('keydown',preventGlobalShortcuts,true);document.addEventListener('dragstart',preventDrag,true);await refreshSystemAccent();checkForUpdates(true);try{if(settings.value.ballPosition)await invoke('restore_ball_position',settings.value.ballPosition);await invoke('main_window_ready');cleanups.push(await getCurrentWindow().onDragDropEvent(e=>{if(e.payload.type==='drop'&&mode.value==='settings'&&section.value==='actions')ingestPaths(e.payload.paths)}));cleanups.push(await listen('compass-show-requested',openCompass));cleanups.push(await listen('compass-settings-requested',openSettings))}catch{}})
onBeforeUnmount(()=>{clearTimeout(idleTimer);document.removeEventListener('keydown',preventGlobalShortcuts,true);document.removeEventListener('dragstart',preventDrag,true);cleanups.forEach(fn=>fn())})
</script>
