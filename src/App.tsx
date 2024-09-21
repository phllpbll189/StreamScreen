import { useState, useEffect } from "react";
import { listen, Event } from '@tauri-apps/api/event';
import "./App.css";

function App() {
  const [currentStatuses, setCurrentStatuses] = useState(["Initializing"]);
  const [frameData, setFrameData] = useState<Uint8Array | null>(null);
  const [deviceId, setDeviceId] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<Uint8Array>('frame_data', (event: Event<Uint8Array>) => {
      const data = event.payload;
      const nullIndex = data.indexOf(0);

      if (nullIndex !== -1) {
        const id = new TextDecoder().decode(data.slice(0, nullIndex));
        setDeviceId(id);
        setFrameData(data.slice(nullIndex + 1));
        console.log("deviceId: ", deviceId);
        console.log("frameData: ", frameData);

      } else {
        setFrameData(data);
      }
    });

    return () => {
      unlisten.then((f: () => void) => f());
    };
  }, []);

  return (
    <div className="container">
      <h1>Welcome to hell</h1>
      {currentStatuses.map((object, i) => <p key={i}>{i}. {object}</p>)}
      {deviceId && <p>Device ID: {deviceId}</p>}
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
