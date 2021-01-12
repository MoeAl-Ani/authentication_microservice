-- MySQL Script generated by MySQL Workbench
-- ma 11. tammikuuta 2021 19.41.33
-- Model: New Model    Version: 1.0
-- MySQL Workbench Forward Engineering

SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0;
SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0;
SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION';

-- -----------------------------------------------------
-- Schema iot
-- -----------------------------------------------------
DROP SCHEMA IF EXISTS `iot` ;

-- -----------------------------------------------------
-- Schema iot
-- -----------------------------------------------------
CREATE SCHEMA IF NOT EXISTS `iot` ;
USE `iot` ;

-- -----------------------------------------------------
-- Table `permission`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `permission` (
  `id` INT NOT NULL AUTO_INCREMENT,
  `name` ENUM('GLOBAL_CREATE', 'GLOBAL_READ', 'GLOBAL_UPDATE', 'GLOBAL_DELETE') NOT NULL,
  PRIMARY KEY (`id`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb4;


-- -----------------------------------------------------
-- Table `language`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `language` (
  `id` INT NOT NULL,
  `language_code` VARCHAR(2) NOT NULL,
  PRIMARY KEY (`id`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb4;


-- -----------------------------------------------------
-- Table `user`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `user` (
  `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
  `first_name` VARCHAR(45) NULL,
  `last_name` VARCHAR(45) NULL,
  `email` VARCHAR(45) NOT NULL,
  `phone_number` VARCHAR(25) NOT NULL,
  `language_id` INT NOT NULL,
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP(),
  PRIMARY KEY (`id`),
  UNIQUE INDEX `email_UNIQUE` (`email` ASC) VISIBLE,
  INDEX `fk_user_language_id_idx` (`language_id` ASC) VISIBLE,
  CONSTRAINT `fk_user_language_id`
    FOREIGN KEY (`language_id`)
    REFERENCES `language` (`id`)
    ON DELETE NO ACTION
    ON UPDATE NO ACTION)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb4;


-- -----------------------------------------------------
-- Table `address`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `address` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  `country` VARCHAR(5) NOT NULL,
  `city` VARCHAR(45) NOT NULL,
  `street` VARCHAR(45) NOT NULL,
  `zip` VARCHAR(45) NULL,
  `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP(),
  PRIMARY KEY (`id`),
  INDEX `address_city_name_idx` (`city` ASC) VISIBLE,
  INDEX `address_street_idx` (`street` ASC) VISIBLE)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb4;


SET SQL_MODE=@OLD_SQL_MODE;
SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS;
SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS;

-- -----------------------------------------------------
-- Data for table `permission`
-- -----------------------------------------------------
START TRANSACTION;
USE `iot`;
INSERT INTO `permission` (`id`, `name`) VALUES (1, 'GLOBAL_CREATE');
INSERT INTO `permission` (`id`, `name`) VALUES (2, 'GLOBAL_READ');
INSERT INTO `permission` (`id`, `name`) VALUES (3, 'GLOBAL_UPDATE');
INSERT INTO `permission` (`id`, `name`) VALUES (4, 'GLOBAL_DELETE');

COMMIT;


-- -----------------------------------------------------
-- Data for table `language`
-- -----------------------------------------------------
START TRANSACTION;
USE `iot`;
INSERT INTO `language` (`id`, `language_code`) VALUES (1, 'EN');
INSERT INTO `language` (`id`, `language_code`) VALUES (2, 'AR');
INSERT INTO `language` (`id`, `language_code`) VALUES (3, 'KU');
INSERT INTO `language` (`id`, `language_code`) VALUES (4, 'FI');

COMMIT;
