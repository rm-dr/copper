import NavBar from "../_navbar";
import SideBar from "../../components/sidebar";
import styles from "./layout.module.scss";

export default function Layout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<>
			<div
				className={styles.navbarContainer}
				style={{
					zIndex: 50,
					position: "relative",
				}}
			>
				<NavBar />
			</div>
			<div className={styles.lowercontent} style={{ zIndex: 20 }}>
				<div className={styles.sidebarContainer}>
					<SideBar />
				</div>
				<div className={styles.contentContainer}>{children}</div>
			</div>
		</>
	);
}
