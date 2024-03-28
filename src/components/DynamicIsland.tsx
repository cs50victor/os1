import {tw} from "../utils/tw"
import { motion } from "framer-motion";

export const IslandStates = ["default", "state_1", "state_2", "state_3"] as const
export type IslandState = typeof IslandStates[number]

const variants : Record<IslandState, Object> = {
  "default": {
    width: "100px",
    height: "30px",
    transition: {
      ease: "easeInOut",
      duration: 0.25,
      property: "all"
    }
  },
  "state_1": {
    width: "300px",
    height: "150px",
    borderRadius: "40px",
    transition: {
      ease: "easeInOut",
      duration: 0.25,
      property: "all"
    }
  },
  "state_2": {
    width: "300px",
    height: "50px",
    borderRadius: "20px",
    transition: {
      ease: "easeInOut",
      duration: 0.25,
      property: "all"
    }
  },
  "state_3": {
    width: "100px",
    height: "50px",
    borderRadius: "20px",
    transition: {
      ease: "easeInOut",
      duration: 0.25,
      property: "all"
    }
  },
};

const stateStyles : Record<IslandState, string> = {
  "default": "align-center",
  "state_1": "align-start",
  "state_2": "align-start p-[8px_16px]",
  "state_3": "align-start p-[8px_16px]",
}

export const DynamicIsland = ({state}: {state: IslandState}) => {

  return (
    <motion.div
      animate={state}
      variants={variants}
      className={tw("shadow-2xl rounded-full bg-black text-white w-fit",stateStyles[state])}
    >
    {/* put your react component here */}
    </motion.div>
  );
};