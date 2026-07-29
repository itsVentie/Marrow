import { currentScreen } from '../../store/state';
import type { Screen } from '../../types';

export function Navigation() {
  const setScreen = (s: Screen) => () => (currentScreen.value = s);

  return (
    <nav className="main-nav glass-panel">
      <button
        className={currentScreen.value === 'dashboard' ? 'active' : ''}
        onClick={setScreen('dashboard')}
      >
        Dashboard
      </button>
      <button
        className={currentScreen.value === 'chat' ? 'active' : ''}
        onClick={setScreen('chat')}
      >
        Chat
      </button>
      <button
        className={currentScreen.value === 'settings' ? 'active' : ''}
        onClick={setScreen('settings')}
      >
        Settings
      </button>
    </nav>
  );
}