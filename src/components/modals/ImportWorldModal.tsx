import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { TauriService } from "../../services/TauriService";

type WorldType = "java" | "xbox360" | "windows64" | null;

export default function ImportWorldModal({
  isOpen,
  onClose,
  playPressSound,
  playBackSound,
  targetInstanceId,
  targetInstanceName,
}: {
  isOpen: boolean;
  onClose: () => void;
  playPressSound: (s?: string) => void;
  playBackSound: (s?: string) => void;
  targetInstanceId: string;
  targetInstanceName: string;
}) {
  const [worldType, setWorldType] = useState<WorldType>(null);
  const [focusIndex, setFocusIndex] = useState(0);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [isImporting, setIsImporting] = useState(false);

  const typeOptions: { id: WorldType; label: string; desc: string }[] = [
    {
      id: "java",
      label: "Java Edition",
      desc: "Import a Java world folder (contains level.dat)",
    },
    {
      id: "xbox360",
      label: "Xbox 360",
      desc: "Import an Xbox 360 STFS save (.bin / .dat)",
    },
    {
      id: "windows64",
      label: "Windows64",
      desc: "Copy an existing LCE save (.ms file or GameHDD folder)",
    },
  ];

  useEffect(() => {
    if (!isOpen) {
      setWorldType(null);
      setFocusIndex(0);
      setStatus("");
      setError("");
      setIsImporting(false);
    }
  }, [isOpen]);

  const handleImport = async () => {
    if (!worldType || !targetInstanceId) return;
    playPressSound();
    setIsImporting(true);
    setError("");
    setStatus("Selecting source...");

    try {
      let inputPath = "";
      let worldName = "";

      if (worldType === "java") {
        setStatus("Selecting Java world folder...");
        const folder = await TauriService.pickFolder();
        if (!folder) {
          setIsImporting(false);
          return;
        }
        inputPath = folder;
        worldName = deriveWorldName(folder);
        setStatus("Converting Java world to LCE...");
      } else if (worldType === "xbox360") {
        setStatus("Selecting Xbox 360 save...");
        const file = await TauriService.pickFile("Select Xbox 360 save", [
          "*.bin",
          "*.dat",
          "*",
        ]);
        if (!file) {
          setIsImporting(false);
          return;
        }
        inputPath = file;
        worldName = deriveWorldName(file);
        setStatus("Converting Xbox 360 save to LCE...");
      } else {
        setStatus("Selecting LCE save folder or .ms file...");
        const picked = await TauriService.pickFile(
          "Select saveData.ms or GameHDD folder",
          ["*.ms", "*"],
        );
        if (!picked) {
          setIsImporting(false);
          return;
        }
        inputPath = picked;
        worldName = picked.endsWith(".ms")
          ? deriveWorldName(picked)
          : deriveWorldName(picked);
        setStatus("Copying LCE save...");
      }

      const instancePath = await TauriService.getInstancePath(targetInstanceId);
      const saveDir = `${instancePath}/Windows64/GameHDD/${worldName}`;
      if (worldType === "windows64") {
        await TauriService.importWorld(inputPath, `${saveDir}/saveData.ms`);
      } else {
        await TauriService.importWorld(inputPath, `${saveDir}/saveData.ms`);
      }

      setStatus(`World imported into "${targetInstanceName}"!`);
      setTimeout(() => {
        onClose();
      }, 2000);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
      setStatus("");
      setIsImporting(false);
    }
  };

  const handleKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      playBackSound();
      if (isImporting) return;
      onClose();
    } else if (
      e.key === "ArrowDown" ||
      e.key === "ArrowUp" ||
      e.key === "Tab"
    ) {
      e.preventDefault();
      const max = typeOptions.length + 1;
      setFocusIndex((prev) => {
        if (e.key === "ArrowUp") return (prev - 1 + max) % max;
        return (prev + 1) % max;
      });
    } else if (e.key === "Enter") {
      if (focusIndex < typeOptions.length) {
        const opt = typeOptions[focusIndex];
        setWorldType(opt.id);
        playPressSound();
      } else {
        onClose();
      }
    }
  };

  useEffect(() => {
    if (!isOpen) return;
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [isOpen, focusIndex, worldType, isImporting]);

  if (!isOpen) return null;
  if (worldType && !isImporting) {
    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 w-screen h-screen z-[100] flex items-center justify-center bg-black/80 backdrop-blur-md"
      >
        <div
          className="relative w-[420px] p-6 flex flex-col items-center shadow-2xl"
          style={{
            backgroundImage: "url('/images/frame_background.png')",
            backgroundSize: "100% 100%",
            imageRendering: "pixelated",
          }}
        >
          <h2 className="text-[#FFFF55] text-2xl mc-text-shadow mb-2 border-b-2 border-[#373737] pb-2 w-full text-center uppercase">
            Import {typeOptions.find((t) => t.id === worldType)?.label}
          </h2>
          <p className="text-white text-sm mc-text-shadow mb-4 text-center">
            Destination:{" "}
            <span className="text-[#FFFF55]">{targetInstanceName}</span>
          </p>
          <p className="text-gray-400 text-xs mc-text-shadow mb-4 text-center">
            {typeOptions.find((t) => t.id === worldType)?.desc}
          </p>

          <div className="flex gap-4 w-full justify-center">
            <button
              onClick={() => {
                setWorldType(null);
                playBackSound();
              }}
              className="w-32 h-10 flex items-center justify-center text-xl text-white mc-text-shadow"
              style={{
                backgroundImage: "url('/images/Button_Background.png')",
                backgroundSize: "100% 100%",
                imageRendering: "pixelated",
              }}
            >
              Back
            </button>
            <button
              onClick={handleImport}
              className="w-40 h-10 flex items-center justify-center text-xl text-white mc-text-shadow hover:text-[#FFFF55]"
              style={{
                backgroundImage: "url('/images/button_highlighted.png')",
                backgroundSize: "100% 100%",
                imageRendering: "pixelated",
              }}
            >
              Select File
            </button>
          </div>
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 w-screen h-screen z-[100] flex items-center justify-center bg-black/80 backdrop-blur-md outline-none border-none"
    >
      <div
        className="relative w-[450px] p-6 flex flex-col items-center shadow-2xl"
        style={{
          backgroundImage: "url('/images/frame_background.png')",
          backgroundSize: "100% 100%",
          imageRendering: "pixelated",
        }}
      >
        {!isImporting ? (
          <>
            <h2 className="text-[#FFFF55] text-2xl mc-text-shadow mb-2 border-b-2 border-[#373737] pb-2 w-full text-center uppercase">
              Import World
            </h2>
            <p className="text-white text-sm mc-text-shadow mb-4 text-center">
              Import into:{" "}
              <span className="text-[#FFFF55]">{targetInstanceName}</span>
            </p>
            <p className="text-gray-400 text-xs mc-text-shadow mb-4 text-center">
              What type of world are you importing?
            </p>

            <div className="w-full mb-4 flex flex-col gap-2">
              {typeOptions.map((opt, i) => {
                const isSelected = worldType === opt.id;
                const isFocused = focusIndex === i;
                return (
                  <div
                    key={opt.id}
                    onClick={() => {
                      playPressSound();
                      setWorldType(opt.id);
                    }}
                    onMouseEnter={() => setFocusIndex(i)}
                    className={`w-full px-4 py-3 cursor-pointer flex flex-col gap-0.5 transition-all outline-none border-none ${
                      isSelected
                        ? "bg-white/15 border-l-4 border-[#FFFF55]"
                        : isFocused
                          ? "bg-white/10 border-l-4 border-[#FFFF55]"
                          : "bg-black/20 hover:bg-black/30 border-l-4 border-transparent"
                    }`}
                    style={{ imageRendering: "pixelated" }}
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-4 h-4 rounded-full border-2 flex items-center justify-center shrink-0 ${
                          isSelected ? "border-[#FFFF55]" : "border-gray-500"
                        }`}
                      >
                        {isSelected && (
                          <div className="w-2 h-2 rounded-full bg-[#FFFF55]" />
                        )}
                      </div>
                      <span className="text-white text-base font-bold mc-text-shadow">
                        {opt.label}
                      </span>
                    </div>
                    <p className="text-gray-400 text-xs ml-7">{opt.desc}</p>
                  </div>
                );
              })}
            </div>

            {error && (
              <div className="text-red-500 text-center mc-text-shadow uppercase text-xs tracking-widest mb-3">
                {error}
              </div>
            )}

            <div className="flex gap-4 w-full justify-center">
              <button
                onMouseEnter={() => setFocusIndex(typeOptions.length)}
                onClick={() => {
                  playBackSound();
                  onClose();
                }}
                className={`w-32 h-10 flex items-center justify-center text-xl mc-text-shadow transition-colors outline-none border-none ${
                  focusIndex === typeOptions.length
                    ? "text-[#FFFF55]"
                    : "text-white"
                }`}
                style={{
                  backgroundImage:
                    focusIndex === typeOptions.length
                      ? "url('/images/button_highlighted.png')"
                      : "url('/images/Button_Background.png')",
                  backgroundSize: "100% 100%",
                  imageRendering: "pixelated",
                }}
              >
                Cancel
              </button>
            </div>
          </>
        ) : (
          <>
            <h2 className="text-[#FFFF55] text-2xl mc-text-shadow mb-4 border-b-2 border-[#373737] pb-2 w-full text-center uppercase">
              Importing World
            </h2>
            <div className="flex flex-col items-center gap-4 py-8">
              <div className="w-12 h-12 border-4 border-[#FFFF55] border-t-transparent rounded-full animate-spin" />
              <p className="text-white text-lg mc-text-shadow text-center">
                {status}
              </p>
            </div>
            {error && (
              <div className="text-red-500 text-center mc-text-shadow uppercase text-xs tracking-widest mb-3">
                {error}
              </div>
            )}
          </>
        )}
      </div>
    </motion.div>
  );
}

function deriveWorldName(inputPath: string): string {
  const name =
    inputPath.replace(/\\/g, "/").split("/").filter(Boolean).pop() ||
    "ImportedWorld";
  return name.replace(/[^a-zA-Z0-9_\- ]/g, "_").slice(0, 64);
}
