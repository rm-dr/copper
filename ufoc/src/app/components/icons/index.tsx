import {
	IconAdjustmentsAlt,
	IconAmpersand,
	IconAnalyze,
	IconArrowRight,
	IconBinary,
	IconBinaryTree,
	IconCheck,
	IconChevronDown,
	IconChevronLeftPipe,
	IconChevronRightPipe,
	IconCircleOff,
	IconCpu,
	IconDatabase,
	IconDatabaseCog,
	IconDatabasePlus,
	IconDatabaseX,
	IconDecimal,
	IconDots,
	IconEdit,
	IconFile,
	IconFileDigit,
	IconFilePlus,
	IconFileUpload,
	IconFileX,
	IconFolder,
	IconFolderPlus,
	IconFolderX,
	IconFolders,
	IconGridPattern,
	IconHexagon,
	IconHexagon3,
	IconHexagonMinus,
	IconHexagonPlus,
	IconLetterCase,
	IconList,
	IconListDetails,
	IconLock,
	IconMenu2,
	IconPlus,
	IconSchema,
	IconSend,
	IconServer2,
	IconSettings2,
	IconSortAscending2,
	IconSortDescending2,
	IconSquareAsterisk,
	IconTableRow,
	IconTrash,
	IconUpload,
	IconUser,
	IconUserOff,
	IconUserPlus,
	IconUsers,
	IconUsersGroup,
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

export const XIconLock = ({ style }: { style?: CSSProperties }) => {
	return <IconLock style={{ ...defaultStyles, ...style }} />;
};

export const XIconNoUser = ({ style }: { style?: CSSProperties }) => {
	return <IconUserOff style={{ ...defaultStyles, ...style }} />;
};

export const XIconUserPlus = ({ style }: { style?: CSSProperties }) => {
	return <IconUserPlus style={{ ...defaultStyles, ...style }} />;
};

export const XIconGroup = ({ style }: { style?: CSSProperties }) => {
	return <IconUsersGroup style={{ ...defaultStyles, ...style }} />;
};

export const XIconUser = ({ style }: { style?: CSSProperties }) => {
	return <IconUser style={{ ...defaultStyles, ...style }} />;
};

export const XIconUsers = ({ style }: { style?: CSSProperties }) => {
	return <IconUsers style={{ ...defaultStyles, ...style }} />;
};

export const XIconArrowRight = ({ style }: { style?: CSSProperties }) => {
	return <IconArrowRight style={{ ...defaultStyles, ...style }} />;
};

export const XIconNoItems = ({ style }: { style?: CSSProperties }) => {
	return <IconCircleOff style={{ ...defaultStyles, ...style }} />;
};

export const XIconAddLeft = ({ style }: { style?: CSSProperties }) => {
	return <IconChevronLeftPipe style={{ ...defaultStyles, ...style }} />;
};

export const XIconAddRight = ({ style }: { style?: CSSProperties }) => {
	return <IconChevronRightPipe style={{ ...defaultStyles, ...style }} />;
};

export const XIconSortUp = ({ style }: { style?: CSSProperties }) => {
	return <IconSortAscending2 style={{ ...defaultStyles, ...style }} />;
};

export const XIconSortDown = ({ style }: { style?: CSSProperties }) => {
	return <IconSortDescending2 style={{ ...defaultStyles, ...style }} />;
};

export const XIconItems = ({ style }: { style?: CSSProperties }) => {
	return <IconListDetails style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrReference = ({ style }: { style?: CSSProperties }) => {
	return <IconAmpersand style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrHash = ({ style }: { style?: CSSProperties }) => {
	return <IconAnalyze style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrText = ({ style }: { style?: CSSProperties }) => {
	return <IconLetterCase style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrBinary = ({ style }: { style?: CSSProperties }) => {
	return <IconBinary style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrBlob = ({ style }: { style?: CSSProperties }) => {
	return <IconFileDigit style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrInt = ({ style }: { style?: CSSProperties }) => {
	return <IconHexagon3 style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrPosInt = ({ style }: { style?: CSSProperties }) => {
	return <IconHexagonPlus style={{ ...defaultStyles, ...style }} />;
};

export const XIconAttrFloat = ({ style }: { style?: CSSProperties }) => {
	return <IconDecimal style={{ ...defaultStyles, ...style }} />;
};

export const XIconDots = ({ style }: { style?: CSSProperties }) => {
	return <IconDots style={{ ...defaultStyles, ...style }} />;
};

export const XIconListArrow = ({ style }: { style?: CSSProperties }) => {
	return <IconChevronDown style={{ ...defaultStyles, ...style }} />;
};

export const XIconRow = ({ style }: { style?: CSSProperties }) => {
	return <IconTableRow style={{ ...defaultStyles, ...style }} />;
};

export const XIconFolderPlus = ({ style }: { style?: CSSProperties }) => {
	return <IconFolderPlus style={{ ...defaultStyles, ...style }} />;
};

export const XIconFolderX = ({ style }: { style?: CSSProperties }) => {
	return <IconFolderX style={{ ...defaultStyles, ...style }} />;
};

export const XIconFolder = ({ style }: { style?: CSSProperties }) => {
	return <IconFolder style={{ ...defaultStyles, ...style }} />;
};

export const XIconFolders = ({ style }: { style?: CSSProperties }) => {
	return <IconFolders style={{ ...defaultStyles, ...style }} />;
};

export const XIconEdit = ({ style }: { style?: CSSProperties }) => {
	return <IconEdit style={{ ...defaultStyles, ...style }} />;
};

export const XIconSettings = ({ style }: { style?: CSSProperties }) => {
	return <IconSettings2 style={{ ...defaultStyles, ...style }} />;
};

export const XIconDatabaseX = ({ style }: { style?: CSSProperties }) => {
	return <IconDatabaseX style={{ ...defaultStyles, ...style }} />;
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
