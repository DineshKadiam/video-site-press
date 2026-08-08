const shortcutsButton = document.querySelector('button.shortcuts');
const shortcutsPanel = document.querySelector('dialog.shortcuts');

shortcutsButton.addEventListener('click', () => {
    if (shortcutsPanel.open) {
        shortcutsPanel.close();
    } else {
        shortcutsPanel.show();
    }
});
