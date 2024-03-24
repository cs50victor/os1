import { useState } from "react";
// import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { Input } from "./components/ui/input";
import { Button } from "./components/ui/button";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  const greet=async()=> {
    setGreetMsg(await invoke("greet", { name }));
  }

  // bg-[#353535] 
  return (
    <div className="h-dvh w-full flex flex-col items-center justify-center">
      <div className="space-y-4">
        <h1 className="text-3xl font-semibold">OS1</h1>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <Input
            id="greet-input"
            className="text-foreground"
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Enter a name..."
          />
          <Button type="submit" className="mt-4">Greet</Button>
        </form>
        <p>{greetMsg}</p>
      </div>
    </div>
  );
}

export default App;
