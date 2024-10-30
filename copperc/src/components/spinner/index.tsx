import { RefreshCw } from "lucide-react";
import styles from "./spinner.module.scss";
import { ReactNode } from "react";

export function Spinner(params: { size?: string }) {
	return (
		<RefreshCw
			size={params.size || "3rem"}
			color="var(--mantine-color-dimmed)"
			className={styles.rotating}
		/>
	);
}

export function Wrapper(params: { children: ReactNode }) {
	return (
		<div
			style={{
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				width: "100%",
				marginTop: "2rem",
				marginBottom: "2rem",
				userSelect: "none",
			}}
		>
			<div
				style={{
					display: "block",
					textAlign: "center",
				}}
			>
				{params.children}
			</div>
		</div>
	);
}
