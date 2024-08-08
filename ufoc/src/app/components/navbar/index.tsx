import styles from "./navbar.module.scss";

const Navbar = () => {
	return (
		<div className={styles.navbar}>
			<div></div>

			<div className={styles.item}>
				<span className={styles.username}>User</span>
			</div>
		</div>
	);
};

export default Navbar;
