function toggle(popup, enabled) {
  if (enabled === false) return;
  if (!popup) return;
  if (popup.opened) popup.close();
  else popup.open();
}

function clickTrigger(popup, wasOpen) {
  if (!popup) return;
  if (wasOpen) popup.close();
  else popup.open();
}
