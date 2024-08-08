import {
	IconAdjustmentsAlt,
	IconBinaryTree,
	IconCheck,
	IconCpu,
	IconDatabase,
	IconDatabaseCog,
	IconDatabasePlus,
	IconFile,
	IconFilePlus,
	IconFileUpload,
	IconFileX,
	IconGridPattern,
	IconHexagon,
	IconHexagonMinus,
	IconList,
	IconMenu2,
	IconPlus,
	IconSchema,
	IconSend,
	IconServer2,
	IconSettings2,
	IconSquareAsterisk,
	IconTrash,
	IconUpload,
	IconX,
} from "@tabler/icons-react";
import { CSSProperties } from "react";

const defaultStyles: CSSProperties | undefined = {
	strokeWidth: "1.5px",
	width: "100%",
	height: "1.5rem",
	margin: "auto",
};

// This module lets us apply default styling to tabler icons,
// and makes it easy to replace our icon provider.

export const XIconSettings = ({ style }: { style?: CSSProperties }) => {
	return <IconSettings2 style={{ ...defaultStyles, ...style }} />;
};

export const XIconDatabasePlus = ({ style }: { style?: CSSProperties }) => {
	return <IconDatabasePlus style={{ ...defaultStyles, ...style }} />;
};

export const XIconDatabaseCog = ({ style }: { style?: CSSProperties }) => {
	return <IconDatabaseCog style={{ ...defaultStyles, ...style }} />;
};

export const XIconDatabase = ({ style }: { style?: CSSProperties }) => {
	return <IconDatabase style={{ ...defaultStyles, ...style }} />;
};

export const XIconGrid = ({ style }: { style?: CSSProperties }) => {
	return <IconGridPattern style={{ ...defaultStyles, ...style }} />;
};

export const XIconList = ({ style }: { style?: CSSProperties }) => {
	return <IconList style={{ ...defaultStyles, ...style }} />;
};

export const XIconPlus = ({ style }: { style?: CSSProperties }) => {
	return <IconPlus style={{ ...defaultStyles, ...style }} />;
};

export const XIconServer = ({ style }: { style?: CSSProperties }) => {
	return <IconServer2 style={{ ...defaultStyles, ...style }} />;
};

export const XIconHex = ({ style }: { style?: CSSProperties }) => {
	return <IconHexagon style={{ ...defaultStyles, ...style }} />;
};

export const XIconPipeline = ({ style }: { style?: CSSProperties }) => {
	return <IconSchema style={{ ...defaultStyles, ...style }} />;
};

export const XIconAdjustments = ({ style }: { style?: CSSProperties }) => {
	return <IconAdjustmentsAlt style={{ ...defaultStyles, ...style }} />;
};

export const XIconHexMinus = ({ style }: { style?: CSSProperties }) => {
	return <IconHexagonMinus style={{ ...defaultStyles, ...style }} />;
};

export const XIconBinaryTree = ({ style }: { style?: CSSProperties }) => {
	return <IconBinaryTree style={{ ...defaultStyles, ...style }} />;
};

export const XIconCheck = ({ style }: { style?: CSSProperties }) => {
	return <IconCheck style={{ ...defaultStyles, ...style }} />;
};

export const XIconFileX = ({ style }: { style?: CSSProperties }) => {
	return <IconFileX style={{ ...defaultStyles, ...style }} />;
};

export const XIconFile = ({ style }: { style?: CSSProperties }) => {
	return <IconFile style={{ ...defaultStyles, ...style }} />;
};

export const XIconFilePlus = ({ style }: { style?: CSSProperties }) => {
	return <IconFilePlus style={{ ...defaultStyles, ...style }} />;
};

export const XIconSend = ({ style }: { style?: CSSProperties }) => {
	return <IconSend style={{ ...defaultStyles, ...style }} />;
};

export const XIconFileUpload = ({ style }: { style?: CSSProperties }) => {
	return <IconFileUpload style={{ ...defaultStyles, ...style }} />;
};

export const XIconUpload = ({ style }: { style?: CSSProperties }) => {
	return <IconUpload style={{ ...defaultStyles, ...style }} />;
};

export const XIconMenu = ({ style }: { style?: CSSProperties }) => {
	return <IconMenu2 style={{ ...defaultStyles, ...style }} />;
};

export const XIconCpu = ({ style }: { style?: CSSProperties }) => {
	return <IconCpu style={{ ...defaultStyles, ...style }} />;
};

export const XIconAst = ({ style }: { style?: CSSProperties }) => {
	return <IconSquareAsterisk style={{ ...defaultStyles, ...style }} />;
};

export const XIconTrash = ({ style }: { style?: CSSProperties }) => {
	return <IconTrash style={{ ...defaultStyles, ...style }} />;
};

export const XIconX = ({ style }: { style?: CSSProperties }) => {
	return <IconX style={{ ...defaultStyles, ...style }} />;
};
