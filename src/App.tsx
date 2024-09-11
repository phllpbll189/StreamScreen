import { useState, useEffect } from "react";
import { listen, Event } from '@tauri-apps/api/event';
import "./App.css";

function App() {

  const [currentStatuses, setCurrentStatuses] = useState(["Initializing"]);
  const [frameData, setFrameData] = useState<Uint8Array | null>(null);

  useEffect(() => {
    const unlisten = listen<Uint8Array>('frame_data', (event: Event<Uint8Array>) => {
      const data = event.payload;
      setFrameData(data);
    });

    return () => {
      unlisten.then((f: () => void) => f());
    };
  }, []);

  return (
    <div className="container">
      <h1>Welcome to hell</h1>
      {currentStatuses.map((object, i) => <p>{i}. {object}</p>)}
      {frameData && (
        <img
          src={URL.createObjectURL(new Blob([frameData], { type: 'image/jpeg' }))}
          alt="Frame"
        />
      )}
    </div>
  );
}

export default App;
