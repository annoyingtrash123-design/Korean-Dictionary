import { useEffect, useState } from 'preact/hooks';
import { db } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { dayNumber } from '../lib/home-day';
import { HomePager } from '../components/TodayCarousel';

const dateLabel = () => new Date().toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });

/** Header of the Home tab (in place of the search bar): "Today · 17 Nov", the local date. */
export function HomeHeader() {
  const [d, setD] = useState(dateLabel);
  useEffect(() => {
    const f = () => setD(dateLabel());
    document.addEventListener('visibilitychange', f);
    return () => document.removeEventListener('visibilitychange', f);
  }, []);
  return (
    <header class="topbar home-head">
      <h1 class="home-title"><span lang="en">Today</span><span class="home-date">· {d}</span></h1>
    </header>
  );
}

/** Home tab: the day's pages (idiom or proverb, three words and two grammar patterns of the curriculum), one per screen. */
export function Home() {
  const [day, setDay] = useState(() => dayNumber());
  // the app can stay open past midnight: pick up the new day when it comes back to the foreground
  useEffect(() => {
    const f = () => setDay(dayNumber());
    document.addEventListener('visibilitychange', f);
    window.addEventListener('focus', f);
    return () => { document.removeEventListener('visibilitychange', f); window.removeEventListener('focus', f); };
  }, []);
  const home = useAsync(() => db.home(day).catch(() => undefined), [day]);
  return <div class="page home">{home.data && <HomePager data={home.data} />}</div>;
}
