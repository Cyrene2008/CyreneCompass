const messages = {
  zh: {
    settings: '设置', backToBall: '返回悬浮球', general: '常规', actions: '罗盘选项', about: '关于',
    generalLead: '控制罗盘的外观、尺寸、行为和启动方式。', theme: '主题', peach: '桃粉', custom: '自定义', accent: '主题色', dark: '深色模式', language: '语言',
    appearance: '尺寸与显示', ballStyle: '悬浮球样式', solid: '单色球体', textBall: '文字球体', iconBall: '图标球体', ballText: '球体文字', ballTextPlaceholder: '最多四个字', image: '图片 / GIF', ballColor: '悬浮球颜色', chooseBallImage: '导入图片 / GIF', cropBallImage: '裁剪为圆形', ballSize: '悬浮球大小', ballOpacity: '悬浮球透明度', compassOpacity: '窗口背景透明度', compassSize: '罗盘窗口大小', uiScale: '页面整体缩放', iconSize: '图标大小', fontSize: '文字大小', fontFamily: '字体',
    behavior: '行为', autoHide: '无操作自动收起', autoHideSeconds: '等待时长', seconds: '秒', startup: '启动', scheduled: '登录时启动（高优先级计划任务）', registry: '普通用户启动项',
    actionsLead: '把任意文件或文件夹拖进页面，Windows 会提供名称、类型与系统图标。', dropTitle: '拖入文件、文件夹或快捷方式', dropHint: '也可以点此选择；系统图标为默认，Fluent 图标可手动覆盖', addAction: '添加空选项', addSubmenu: '创建子菜单', administrator: '管理员', chooseIcon: '选择图标', customIcon: '选择自定义图片', useSystemIcon: '恢复系统图标', clear: '清空', target: '路径 / URI / URL / PowerShell 脚本', label: '显示文字', type: '类型',
    iconLibrary: 'Fluent 图标库', searchIcon: '搜索图标', cancel: '取消', confirm: '确认', close: '关闭', apply: '确认应用', enterSubmenu: '进入子菜单', submenuType: '子菜单', scriptType: '脚本', shortcutType: '快捷方式', folderType: '文件夹', fileType: '文件', actionType: '动作', menuFull: '每级菜单最多只能放置 8 个选项。',
    aboutLead: 'Fluent quick launcher for Windows 10+。', version: '版本', quit: '退出程序', quit1: '退出 Cyreneの罗盘？', quit2: '您真的要不退出Cyreneの罗盘吗？', quit3: '最后一次确认：退出后悬浮球与托盘都会关闭。', continueQuit: '继续退出', stay: '留在这里', executed: '已执行', startupUpdated: '启动设置已更新', inspectFailed: '无法解析拖入对象', emptyAction: '新选项', newMenu: '新子菜单'
  },
  en: {
    settings: 'Settings', backToBall: 'Back to floating ball', general: 'General', actions: 'Compass items', about: 'About',
    generalLead: 'Control appearance, sizing, behavior, and startup.', theme: 'Theme', peach: 'Peach pink', custom: 'Custom', accent: 'Accent color', dark: 'Dark mode', language: 'Language',
    appearance: 'Size and display', ballStyle: 'Floating ball style', solid: 'Solid ball', textBall: 'Text ball', iconBall: 'Icon ball', ballText: 'Ball text', ballTextPlaceholder: 'Up to 4 characters', image: 'Image / GIF', ballColor: 'Ball color', chooseBallImage: 'Import image / GIF', cropBallImage: 'Crop to circle', ballSize: 'Ball size', ballOpacity: 'Ball opacity', compassOpacity: 'Window background opacity', compassSize: 'Compass window size', uiScale: 'UI scale', iconSize: 'Icon size', fontSize: 'Text size', fontFamily: 'Font',
    behavior: 'Behavior', autoHide: 'Auto-collapse when idle', autoHideSeconds: 'Idle timeout', seconds: 'sec', startup: 'Startup', scheduled: 'Start at login (high-priority task)', registry: 'Standard user startup entry',
    actionsLead: 'Drop any file or folder here. Windows supplies its display name, type, and system icon.', dropTitle: 'Drop files, folders, or shortcuts', dropHint: 'Or click to browse. System icons are default; Fluent icons are optional overrides.', addAction: 'Add empty item', addSubmenu: 'Create submenu', administrator: 'Administrator', chooseIcon: 'Choose icon', customIcon: 'Choose custom image', useSystemIcon: 'Restore system icon', clear: 'Clear', target: 'Path / URI / URL / PowerShell script', label: 'Display label', type: 'Type',
    iconLibrary: 'Fluent icon library', searchIcon: 'Search icons', cancel: 'Cancel', confirm: 'Confirm', close: 'Close', apply: 'Apply', enterSubmenu: 'Open submenu', submenuType: 'Submenu', scriptType: 'Script', shortcutType: 'Shortcut', folderType: 'Folder', fileType: 'File', actionType: 'Action', menuFull: 'Each menu can contain up to 8 items.',
    aboutLead: 'Fluent quick launcher for Windows 10+.', version: 'Version', quit: 'Quit application', quit1: 'Quit Cyrene Compass?', quit2: 'Do you really not want to keep Cyrene Compass running?', quit3: 'Final confirmation: the floating ball and tray icon will close.', continueQuit: 'Continue quitting', stay: 'Stay', executed: 'Executed', startupUpdated: 'Startup settings updated', inspectFailed: 'Could not inspect dropped item', emptyAction: 'New item', newMenu: 'New submenu'
  }
}

export function translate(language, key) { return messages[language]?.[key] ?? messages.zh[key] ?? key }
