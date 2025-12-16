import { useContext } from "react";
import { AppStateContext } from "./app-state";

export const useAppState = () => useContext(AppStateContext);
