import NavBar from "../components/navbar";
import SideBar from "../components/sidebar";
import styles from "./layout.module.scss";

export default function Layout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<>
			<div className={styles.navbarContainer}>
				<NavBar />
			</div>
			<div className={styles.lowercontent}>
				<div className={styles.sidebarContainer}>
					<SideBar />
				</div>
				<div className={styles.contentContainer}>{children}</div>
			</div>
		</>
	);
}
