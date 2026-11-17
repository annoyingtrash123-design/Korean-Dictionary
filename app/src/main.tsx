import { render } from 'preact';
import { App } from './App';
import { installErrorCapture } from './lib/errlog';
import './styles.css';

installErrorCapture();
render(<App />, document.getElementById('app')!);
import { installBackButton } from './lib/native';
void installBackButton();
