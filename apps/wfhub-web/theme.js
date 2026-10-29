const savedTheme = localStorage.getItem('wfhub-theme');
function setTheme(theme) { document.documentElement.dataset.theme = theme; localStorage.setItem('wfhub-theme', theme); document.querySelectorAll('.theme').forEach(button => button.setAttribute('aria-label', `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`)); }
setTheme(savedTheme || (matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'));
document.querySelectorAll('.theme').forEach(button => button.addEventListener('click', () => setTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark')));
